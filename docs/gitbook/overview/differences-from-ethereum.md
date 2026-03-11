---
icon: file-plus-minus
---

# Differences from Ethereum

## Overview

The [Seismic EVM](https://github.com/SeismicSystems/seismic-revm/blob/c29f4ea0681f09fb1e9998fa16568d4979c47ee3/README.md) is approximately a superset of the EVM

### What's the same

- Transaction construction and serialization identical to Ethereum (with one new transaction type)
- Address generation, gas estimation, and signing work the same as Ethereum
- [Almost all](differences-from-ethereum.md#rpc-compatibility) RPC methods are identical to reth
- Standard Solidity bytecode will behave identically on Seismic, with [minor exceptions](../reference/opcodes.md#sstore-sload-standard) (e.g., `SSTORE` reverts on shielded slots)
- Seismic supports all of Ethereum's opcodes & precompiles&#x20;
- EIP-1559 transactions follow standard EIP-1559 fee rules. Seismic transactions (type `0x4A`) use legacy fee pricing
- Seismic will produce empty blocks when there are no pending transactions

### Key differences

- **Shielded storage**: Solidity contracts can store private data on-chain
- **Runs in a TEE**: Seismic nodes must run in Trusted Execution Environments
- **Seismic transaction:** We added a new transaction type that allows you to encrypt your calldata

## EVM Compatibility

### Opcodes

- [`CLOAD`](../reference/opcodes.md#cload) – load shielded data from storage
- [`CSTORE`](../reference/opcodes.md#cstore) – write shielded data to storage
- [`TIMESTAMP_MS`](../reference/opcodes.md#timestamp_ms) – get the block timestamp in milliseconds. Note that `block.timestamp` still returns seconds, matching standard Solidity. Use `block.timestamp_ms` for millisecond precision

### Seismic transaction

The transaction with type `0x4a` allows users to encrypt their calldata. These otherwise work just like legacy transactions. We also support the other standard Ethereum transaction types (Legacy, EIP-1559, EIP-2930, EIP-4844, EIP-7702)

### Precompiles

All standard Ethereum precompiles are still available. Seismic added 6 new [precompiles](../reference/precompiles.md) to our EVM:

- [**RNG `0x64`**](../reference/precompiles.md#rng-0x64) — securely generate a random number
- [**ECDH `0x65`**](../reference/precompiles.md#ecdh-0x65) — Elliptic Curve Diffie-Hellman, for generating a shared secret given a public key and secret key
- [**AES-GCM Encrypt `0x66`**](../reference/precompiles.md#aes-gcm-encrypt-0x66) — encrypt data with AES-GCM
- [**AES-GCM Decrypt `0x67`**](../reference/precompiles.md#aes-gcm-decrypt-0x67) — decrypt data with AES-GCM
- [**HKDF `0x68`**](../reference/precompiles.md#hkdf-0x68) — derive cryptographic keys from a parent key
- [**secp256k1 Sign `0x69`**](../reference/precompiles.md#secp256k1-sign-0x69) — sign a message given a secret key

### Staking

Seismic uses the same staking contract as Ethereum, which is hardcoded into our Genesis block at address `0x00000000219ab540356cbb839cbe05303d7705fa`&#x20;

### Block times

We will often produce multiple blocks in the same second, yet Ethereum's block timestamps are expressed in terms of unix seconds. Our solution to this:

- Block headers and the EVM use timestamps in milliseconds internally
- In Seismic Solidity, `block.timestamp` returns unix seconds, just like in standard Solidity. We added `block.timestamp_ms` which returns unix milliseconds. `block.timestamp_seconds` is an alias for `block.timestamp`

### RPC compatibility

We support almost every RPC endpoint in Reth, and have added a few more of our own. See the full [RPC Methods](../reference/rpc-methods/) reference for details.

**Seismic-specific methods:**

- [`seismic_getTeePublicKey`](../reference/rpc-methods/seismic-get-tee-public-key.md) — returns the TEE's encryption public key for ECDH key exchange

**Modified Ethereum methods:**

- [`eth_call`](../reference/rpc-methods/eth-call.md) — zeroes the `from` field on unsigned calls; supports ["signed reads"](../reference/seismic-transaction/signed-reads.md) via type `0x4A`
- [`eth_sendRawTransaction`](../reference/rpc-methods/eth-send-raw-transaction.md) — accepts Seismic transaction type `0x4A` with encrypted calldata
- [`eth_getStorageAt`](../reference/rpc-methods/eth-get-storage-at.md) — returns zero for shielded slots, making them indistinguishable from uninitialized storage
- Tracing endpoints are currently disabled. We plan to re-enable them with shielded data redacted from traces
