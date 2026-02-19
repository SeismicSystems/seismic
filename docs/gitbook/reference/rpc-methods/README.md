---
icon: satellite-dish
---

# RPC Methods

Seismic nodes expose a JSON-RPC API that is almost entirely compatible with standard Ethereum RPC. You can use the same tools (curl, cast, ethers.js, viem) with the same methods you already know.

This section documents the methods most relevant to Seismic developers — including Seismic-specific methods and standard Ethereum methods that behave differently on Seismic.

## Seismic-Specific Methods

| Method                                                     | Description                                                                                            |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| [`seismic_getTeePublicKey`](seismic-get-tee-public-key.md) | Returns the TEE's encryption public key, used for ECDH key exchange when building Seismic transactions |

## Modified Ethereum Methods

These standard Ethereum methods work on Seismic but have important behavioral differences:

| Method                                                  | Seismic Behavior                                                                                                                       |
| ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| [`eth_call`](eth-call.md)                               | Zeroes the `from` field on unsigned calls to prevent caller-dependent behavior leaks. Supports signed reads via Seismic tx type `0x4A` |
| [`eth_sendRawTransaction`](eth-send-raw-transaction.md) | Accepts Seismic transaction type `0x4A` with encrypted calldata and `SeismicElements` metadata                                         |
| [`eth_getStorageAt`](eth-get-storage-at.md)             | Fails with an error if the requested storage slot holds shielded (private) data                                                        |

## Standard Ethereum Methods

These methods work identically to their Ethereum counterparts:

| Method                                   | Description                                       |
| ---------------------------------------- | ------------------------------------------------- |
| [`eth_blockNumber`](eth-block-number.md) | Returns the number of the most recent block       |
| [`eth_getBalance`](eth-get-balance.md)   | Returns the balance of an account in wei          |
| [`eth_chainId`](eth-chain-id.md)         | Returns the chain ID (`0x1404` / 5124 on testnet) |
| [`net_version`](net-version.md)          | Returns the current network ID                    |

## RPC Endpoints

| Network | URL                                 |
| ------- | ----------------------------------- |
| Testnet | `https://gcp-0.seismictest.net/rpc` |
| Devnet  | `https://node-2.seismicdev.net/rpc` |

## Quick Test

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
```

{% hint style="info" %}
Seismic supports almost every RPC endpoint available in [Reth](https://reth.rs/). Only tracing endpoints are modified (shielded data is removed from traces). See [Differences from Ethereum](../../overview/differences-from-ethereum.md#rpc-compatibility) for details.
{% endhint %}
