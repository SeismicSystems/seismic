---
icon: satellite-dish
---

# RPC Methods

Seismic nodes expose a JSON-RPC API that is almost entirely compatible with standard Ethereum RPC. You can use the same tools (curl, cast, ethers.js, viem) with the same methods you already know. For full Seismic support (encrypted calldata, signed reads, shielded types), use one of our [client libraries](../../clients/README.md) or [`scast`](../../getting-started/development-toolkit.md) (part of the seismic-foundry toolset).

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
| [`eth_getStorageAt`](eth-get-storage-at.md)             | Returns zero for shielded (private) storage slots, as if the slot were uninitialized                                                   |

## Blocked Tracing Endpoints

All tracing and debug endpoints (`debug_traceTransaction`, `debug_traceCall`, `trace_transaction`, `trace_block`, etc.) are blocked on Seismic. Execution traces would reveal shielded values — opcodes, memory, and stack contents could expose private data that contracts are designed to protect. We plan to eventually support these endpoints with private data redacted from the results, but this is not available today.
