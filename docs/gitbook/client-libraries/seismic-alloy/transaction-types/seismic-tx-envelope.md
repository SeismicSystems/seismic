---
description: Signed transaction wrapper supporting all Ethereum and Seismic types
icon: envelope
---

# SeismicTxEnvelope

An enum wrapping all supported signed transaction types, including the Seismic type (`0x4A`). This is the final form of a transaction before RLP encoding and network submission.

## Overview

`SeismicTxEnvelope` extends Alloy's standard `TxEnvelope` with a `Seismic` variant for type `0x4A` transactions. It wraps a signed transaction (any supported type) into a unified enum that can be RLP-encoded and broadcast to the network. The provider produces a `SeismicTxEnvelope` internally when you call `send_transaction()`.

## Definition

```rust
pub enum SeismicTxEnvelope {
    Legacy(Signed<TxLegacy>),
    Eip2930(Signed<TxEip2930>),
    Eip1559(Signed<TxEip1559>),
    Eip4844(Signed<TxEip4844Variant>),
    Eip7702(Signed<TxEip7702>),
    Seismic(Signed<TxSeismic>),
}
```

## Variants

| Variant   | Type Code | Inner Type                 | Description                        |
| --------- | --------- | -------------------------- | ---------------------------------- |
| `Legacy`  | `0x00`    | `Signed<TxLegacy>`         | Pre-EIP-2718 legacy transaction    |
| `Eip2930` | `0x01`    | `Signed<TxEip2930>`        | Access list transaction (EIP-2930) |
| `Eip1559` | `0x02`    | `Signed<TxEip1559>`        | Fee market transaction (EIP-1559)  |
| `Eip4844` | `0x03`    | `Signed<TxEip4844Variant>` | Blob transaction (EIP-4844)        |
| `Eip7702` | `0x04`    | `Signed<TxEip7702>`        | Set EOA account code (EIP-7702)    |
| `Seismic` | `0x4A`    | `Signed<TxSeismic>`        | Encrypted Seismic transaction      |

## The `Signed<T>` Wrapper

Each variant contains a `Signed<T>` which pairs the transaction body with its ECDSA signature:

```rust
pub struct Signed<T> {
    tx: T,
    signature: PrimitiveSignature,
    hash: B256,  // Cached transaction hash
}
```

| Field       | Type                 | Description                                                         |
| ----------- | -------------------- | ------------------------------------------------------------------- |
| `tx`        | `T`                  | The unsigned transaction body                                       |
| `signature` | `PrimitiveSignature` | ECDSA signature (v, r, s)                                           |
| `hash`      | `B256`               | Precomputed transaction hash (keccak256 of the signed RLP encoding) |

## How Signed Transactions Are Created

The signing flow for a Seismic transaction:

```
1. TxSeismic is fully populated (fields + encrypted calldata)
       |
       v
2. Compute signing hash:
   - message_version == 0: keccak256(RLP(unsigned_fields))
   - message_version >= 2: EIP-712 signing hash
       |
       v
3. ECDSA sign with wallet private key -> (v, r, s)
       |
       v
4. Wrap: Signed { tx: TxSeismic, signature, hash }
       |
       v
5. Wrap: SeismicTxEnvelope::Seismic(signed_tx)
       |
       v
6. RLP encode with 0x4A type prefix -> raw bytes
       |
       v
7. Broadcast via eth_sendRawTransaction
```

## Examples

### Matching on Envelope Variants

```rust
use seismic_alloy::prelude::*;

fn inspect_envelope(envelope: &SeismicTxEnvelope) {
    match envelope {
        SeismicTxEnvelope::Legacy(signed) => {
            println!("Legacy tx, nonce: {}", signed.tx().nonce);
        }
        SeismicTxEnvelope::Eip1559(signed) => {
            println!("EIP-1559 tx, max fee: {}", signed.tx().max_fee_per_gas);
        }
        SeismicTxEnvelope::Seismic(signed) => {
            let tx = signed.tx();
            println!("Seismic tx (0x4A):");
            println!("  nonce: {}", tx.nonce);
            println!("  to: {:?}", tx.to);
            println!("  signed_read: {}", tx.seismic_elements.signed_read);
            println!("  expires_at: {}", tx.seismic_elements.expires_at_block);
        }
        _ => println!("Other transaction type"),
    }
}
```

### Accessing the Transaction Hash

```rust
use seismic_alloy::prelude::*;

let envelope: SeismicTxEnvelope = /* from provider */;

// Get the transaction hash (works for all variants)
let tx_hash = envelope.tx_hash();
println!("Transaction hash: {:?}", tx_hash);
```

### Accessing the Signature

```rust
let envelope = SeismicTxEnvelope::Seismic(signed_tx);

if let SeismicTxEnvelope::Seismic(signed) = &envelope {
    let sig = signed.signature();
    println!("v: {}", sig.v());
    println!("r: {:?}", sig.r());
    println!("s: {:?}", sig.s());
}
```

## Relationship to Alloy's TxEnvelope

Alloy defines a standard `TxEnvelope` for Ethereum transaction types (Legacy, EIP-2930, EIP-1559, EIP-4844, EIP-7702). `SeismicTxEnvelope` extends this with the `Seismic` variant:

| Alloy `TxEnvelope` | `SeismicTxEnvelope`     |
| ------------------ | ----------------------- |
| `Legacy`           | `Legacy`                |
| `Eip2930`          | `Eip2930`               |
| `Eip1559`          | `Eip1559`               |
| `Eip4844`          | `Eip4844`               |
| `Eip7702`          | `Eip7702`               |
| --                 | `Seismic` (type `0x4A`) |

This design allows seismic-alloy to handle all standard Ethereum transactions plus the Seismic type through a single unified type. Standard Alloy operations (sending legacy or EIP-1559 transactions) work normally.

## RLP Encoding

The envelope uses EIP-2718 typed transaction encoding:

- **Legacy transactions**: RLP-encoded directly (no type prefix)
- **All other types**: `type_byte || RLP(signed_fields)`

For the Seismic variant:

```
0x4A || RLP([
    chain_id,
    nonce,
    gas_price,
    gas_limit,
    to,
    value,
    input,
    encryption_pubkey,
    encryption_nonce,
    message_version,
    recent_block_hash,
    expires_at_block,
    signed_read,
    v, r, s
])
```

## Trait Implementations

`SeismicTxEnvelope` implements:

| Trait                             | Description                                   |
| --------------------------------- | --------------------------------------------- |
| `Typed2718`                       | EIP-2718 type byte access                     |
| `Encodable2718` / `Decodable2718` | EIP-2718 RLP encoding/decoding                |
| `Transaction`                     | Common transaction field accessors            |
| `From<Signed<TxSeismic>>`         | Convert a signed Seismic tx into the envelope |
| `From<TxEnvelope>`                | Convert standard Alloy envelopes              |

## Notes

- **Internal type.** Most users do not construct `SeismicTxEnvelope` directly -- the provider and wallet handle signing and wrapping automatically.
- **Unified handling.** RPC response parsing and block processing use `SeismicTxEnvelope` to handle all transaction types in a single code path.
- **Hash caching.** The `Signed<T>` wrapper caches the transaction hash, so repeated access is efficient.
- **Recoverable signature.** The ECDSA signature in `Signed<T>` is recoverable, allowing the sender's address to be derived from the hash and signature.

## See Also

- [TxSeismic](tx-seismic.md) -- The unsigned Seismic transaction type inside the `Seismic` variant
- [TxSeismicElements](tx-seismic-elements.md) -- Encryption metadata within `TxSeismic`
- [Transaction Types Overview](./) -- All transaction types and their relationships
- [Shielded Calls](../contract-interaction/shielded-calls.md) -- How transactions are built and sent
