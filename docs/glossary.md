# Seismic Concepts Glossary <!-- omit in toc -->

- [Shielded State / FlaggedStorage](#shielded-state--flaggedstorage)
- [Shielded Types](#shielded-types)
- [Mercury Spec](#mercury-spec)
- [TxSeismic](#txseismic)
- [TEE Integration](#tee-integration)
- [SeismicHost](#seismichost)


## Shielded State / FlaggedStorage

Every storage slot is a tuple `(value, is_private)`, called `FlaggedStorage`. Private slots read as 0 via RPC (`eth_getStorageAt`), indistinguishable from uninitialized storage. Inside the EVM, only `CLOAD`/`CSTORE` opcodes can access them — `SLOAD`/`SSTORE` on a private slot revert with a catchable `InvalidPrivateStorageAccess` reason. This allows contracts to keep secrets on-chain that must be accessed via contract logic rather than direct reads. Authentication/Authorization libraries such as OpenZeppelin's AccessControl can thus be used to gate READ access to private state, unlike in a standard EVM where all state is public.

```rust
struct FlaggedStorage {
    value: U256,
    is_private: bool,
}
```

This type flows through: alloy-core → revm (journal) → trie (merkle encoding) → reth (database, RPC).

## Shielded Types

The Seismic Solidity compiler adds `suint`, `sint`, `sbool`, `saddress`, `sbytes` — confidential counterparts of standard Solidity types. They compile down to `CLOAD`/`CSTORE` instead of `SLOAD`/`SSTORE`. They otherwise behave exactly like their public counterparts in terms of arithmetic, ABI encoding, etc. This allows developers to easily write contracts with private state without needing to manage the `(value, is_private)` tuple manually.

See the [Seismic Solidity docs](gitbook/seismic-solidity/README.md) for the full spec (storage behavior, restrictions, casting, collections, best practices).

## Mercury Spec

Seismic's name for the modified EVM specification. Implemented primarily in seismic-revm. Adds:
- **Opcodes**: CLOAD (0xB0), CSTORE (0xB1), TIMESTAMPMS (0x4B — block timestamp in milliseconds)
- **Precompiles**: RNG (0x64), ECDH (0x65), AES-GCM Encrypt (0x66), AES-GCM Decrypt (0x67), HKDF (0x68), secp256k1 Sign (0x69)
- **FlaggedStorage access rules** (slot state `(value, is_private)`):

  |             | (0, public)  | (x, public) | (0, private) | (x, private) |
  | ----------- | ------------ | ----------- | ------------ | ------------ |
  | `SLOAD`     | 0            | x           | revert       | revert       |
  | `CLOAD`     | 0            | x           | 0            | x            |
  | `SSTORE(y)` | (y, public)  | (y, public) | revert       | revert       |
  | `CSTORE(y)` | (y, private) | revert      | (y, private) | (y, private) |

- **Flat gas costs** for CLOAD/CSTORE: no cold/warm or value-transition variation, and no refunds, so gas cannot leak shielded values

See the [Opcodes reference](gitbook/reference/opcodes.md) for details.

## TxSeismic

Transaction type ID `74` (0x4A). Standard EVM transaction with encrypted calldata. Encryption uses ECDH + AEAD. Additional fields:

```rust
struct TxSeismicElements {
    encryption_pubkey: PublicKey,  // 33-byte compressed secp256k1
    encryption_nonce: U96,        // AEAD nonce (12 bytes)
    message_version: u8,
    recent_block_hash: B256,      // prevents replay across forks
    expires_at_block: u64,        // expiration block number
    signed_read: bool,
}
```

Defined in seismic-alloy, consumed by seismic-evm, seismic-reth, and seismic-foundry. See [The Seismic Transaction](gitbook/reference/seismic-transaction/README.md) for the encryption scheme and transaction lifecycle.

## TEE Integration

Nodes run inside a Trusted Execution Environment. The TEE holds the decryption key. Users encrypt calldata with the network's public key (fetched via `seismic_getTeePublicKey` RPC). Encrypted calldata is decrypted in-enclave at execution time: in the block executor for state-changing transactions (so they stay encrypted in the mempool and over gossip), and at the RPC layer for reads (`eth_call` / signed reads).

## SeismicHost

Trait in seismic-revm extending the standard EVM Host with confidential storage operations. Implemented by the journal/state layer.
