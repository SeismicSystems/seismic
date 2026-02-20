---
description: Seismic transaction types and data structures in seismic-alloy
icon: arrows-split-up-and-left
---

# Transaction Types

Core data structures for building, signing, encrypting, and submitting Seismic transactions.

## Overview

seismic-alloy defines several transaction types that extend Alloy's standard Ethereum types with Seismic-specific encryption and privacy features. These types live in the `seismic-alloy-consensus` crate and are re-exported through the prelude.

## Type Summary

| Type                                                      | Description                                 | Use Case                                                             |
| --------------------------------------------------------- | ------------------------------------------- | -------------------------------------------------------------------- |
| [`TxSeismic`](tx-seismic.md)                              | Encrypted Seismic transaction (type `0x4A`) | Privacy-preserving contract calls and state changes                  |
| [`TxSeismicElements`](tx-seismic-elements.md)             | Encryption metadata and security parameters | Attached to every `TxSeismic` for key exchange, nonce, and expiry    |
| [`TxSeismicMetadata`](tx-seismic-metadata.md)             | Additional Authenticated Data (AAD) context | AEAD encryption binding -- ensures ciphertext is tied to tx params   |
| [`TxLegacyFields`](tx-seismic-metadata.md#txlegacyfields) | Standard EVM transaction fields             | Subset of tx fields used in AAD construction                         |
| [`SeismicTxEnvelope`](seismic-tx-envelope.md)             | Signed transaction wrapper enum             | Network submission -- wraps all supported tx types including Seismic |

## Transaction Lifecycle

```
TransactionRequest           Build with seismic_foundry_tx_builder()
       |
       v
SeismicTransactionRequest    Mark as seismic with .seismic()
       |
       v
TxSeismic                    Filled by filler pipeline (nonce, gas, elements)
       |
       v
Signed<TxSeismic>            Signed with wallet private key
       |
       v
SeismicTxEnvelope::Seismic   Wrapped in envelope for RLP encoding
       |
       v
Network                      Broadcast via send_raw_transaction
```

## Relationship Between Types

```
TxSeismic
├── chain_id, nonce, gas_price, gas_limit  (standard fields)
├── to, value, input                        (standard fields)
└── seismic_elements: TxSeismicElements
    ├── encryption_pubkey      (secp256k1 compressed, 33 bytes)
    ├── encryption_nonce       (12-byte AES-GCM nonce)
    ├── message_version        (0=RLP, >=2=EIP-712)
    ├── recent_block_hash      (freshness proof)
    ├── expires_at_block       (expiration block number)
    └── signed_read            (true for eth_call reads)

TxSeismicMetadata (for AAD construction)
├── sender: Address
├── legacy_fields: TxLegacyFields
│   ├── chain_id
│   ├── nonce
│   ├── to
│   └── value
└── seismic_elements: TxSeismicElements (same as above)

SeismicTxEnvelope (signed wrapper)
├── Legacy(Signed<TxLegacy>)
├── Eip2930(Signed<TxEip2930>)
├── Eip1559(Signed<TxEip1559>)
├── Eip4844(Signed<TxEip4844Variant>)
├── Eip7702(Signed<TxEip7702>)
└── Seismic(Signed<TxSeismic>)       <-- Seismic type 0x4A
```

## Crate Location

These types are defined in `seismic-alloy-consensus` and re-exported through `seismic-alloy-prelude`:

```rust
use seismic_alloy::prelude::*;
// or directly:
use seismic_alloy_consensus::{TxSeismic, TxSeismicElements, TxSeismicMetadata};
```

## Navigation

| Page                                        | Description                                                     |
| ------------------------------------------- | --------------------------------------------------------------- |
| [TxSeismic](tx-seismic.md)                  | The core Seismic transaction struct with all fields and methods |
| [TxSeismicElements](tx-seismic-elements.md) | Encryption parameters, builder methods, and crypto helpers      |
| [TxSeismicMetadata](tx-seismic-metadata.md) | AAD generation for AEAD encryption                              |
| [SeismicTxEnvelope](seismic-tx-envelope.md) | Signed transaction wrapper for network submission               |

## See Also

- [Contract Interaction](../contract-interaction/) -- How to use these types in practice
- [Shielded Calls](../contract-interaction/shielded-calls.md) -- Building transactions with `.seismic()`
- [Encryption](../provider/encryption.md) -- How encryption uses these types
