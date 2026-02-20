---
description: Transaction type byte for Seismic transactions
icon: code
---

# SEISMIC_TX_TYPE

Protocol constant representing the transaction type byte for Seismic custom transactions.

## Overview

`SEISMIC_TX_TYPE` is the EIP-2718 transaction type identifier used by Seismic's custom transaction format. This byte prefixes all serialized Seismic transactions.

## Value

| Representation | Value |
|----------------|-------|
| Hexadecimal | `0x4A` |
| Decimal | `74` |

## Definition

```python
SEISMIC_TX_TYPE: int = 0x4A
```

## EIP-2718 Context

EIP-2718 introduced typed transaction envelopes to Ethereum. Transaction types are identified by a single byte prefix:

| Type | Description |
|------|-------------|
| `0x00` | Legacy transactions (pre-EIP-2718) |
| `0x01` | EIP-2930 (Access lists) |
| `0x02` | EIP-1559 (Dynamic fee) |
| `0x03` | EIP-4844 (Blob transactions) |
| `0x04` | EIP-7702 (Set EOA account code) |
| `0x4A` | **Seismic custom transactions** |

## Transaction Format

A serialized Seismic transaction has the format:

```
[0x4A] + RLP([chainId, nonce, gasPrice, gas, to, value, data, ...])
```

Where `0x4A` is the `SEISMIC_TX_TYPE` prefix.

## Protocol Notes

- The SDK automatically sets this type for all shielded writes
- Seismic nodes recognize and process transactions with this type
- Standard Ethereum nodes will reject transactions with unknown types
- This type is part of the Seismic protocol specification

## See Also

- [UnsignedSeismicTx](../api-reference/transaction-types/unsigned-seismic-tx.md) - Seismic transaction structure
- [sign_seismic_tx_eip712](../api-reference/eip712/sign-seismic-tx-eip712.md) - Transaction signing
- [ChainConfig](chain-config.md) - Chain configuration with chain IDs
- [Shielded Write Guide](../shielded-write.md) - Guide to sending Seismic transactions
