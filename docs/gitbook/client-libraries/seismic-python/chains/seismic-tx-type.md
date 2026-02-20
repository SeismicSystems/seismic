---
description: Transaction type byte for Seismic transactions
icon: code
---

# SEISMIC_TX_TYPE

Protocol constant representing the transaction type byte for Seismic custom transactions.

## Overview

`SEISMIC_TX_TYPE` is the EIP-2718 transaction type identifier used by Seismic's custom transaction format. This byte prefixes all serialized Seismic transactions.

## Definition

```python
SEISMIC_TX_TYPE: int = 0x4A
```

## Value

| Representation | Value |
|----------------|-------|
| Hexadecimal | `0x4A` |
| Decimal | `74` |
| Binary | `01001010` |

## Purpose

The transaction type byte serves several purposes:

1. **EIP-2718 Compliance** - Identifies this as a custom transaction type
2. **Protocol Recognition** - Allows nodes to distinguish Seismic transactions from standard Ethereum transactions
3. **Serialization Format** - Prefixes the RLP-encoded transaction payload

## Usage

### Automatic Usage

The SDK handles transaction type internally. You typically don't need to reference this constant directly:

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))
w3 = SEISMIC_TESTNET.wallet_client(pk)

# The SDK automatically sets the transaction type
contract = w3.seismic.contract(address="0x...", abi=[...])
tx_hash = contract.swrite.my_function(42)  # Uses SEISMIC_TX_TYPE internally
```

### Manual Access

If you need to inspect or validate transaction types:

```python
from seismic_web3 import SEISMIC_TX_TYPE

print(f"Seismic transaction type: {SEISMIC_TX_TYPE}")        # 74
print(f"Seismic transaction type: {hex(SEISMIC_TX_TYPE)}")  # 0x4a
```

### Transaction Serialization

When serializing Seismic transactions, the type byte is prepended:

```python
from seismic_web3 import SEISMIC_TX_TYPE

# Conceptual example (SDK handles this automatically)
def serialize_seismic_tx(tx_data: bytes) -> bytes:
    """Serialize a Seismic transaction with type prefix."""
    return bytes([SEISMIC_TX_TYPE]) + tx_data
```

### Transaction Type Validation

When parsing raw transactions, you can check the type:

```python
from seismic_web3 import SEISMIC_TX_TYPE

def is_seismic_transaction(raw_tx: bytes) -> bool:
    """Check if a raw transaction is a Seismic transaction."""
    if len(raw_tx) == 0:
        return False
    return raw_tx[0] == SEISMIC_TX_TYPE
```

## Examples

### Checking Transaction Type

```python
from seismic_web3 import SEISMIC_TX_TYPE, SEISMIC_TESTNET

w3 = SEISMIC_TESTNET.public_client()

# Get a transaction from the chain
tx = w3.eth.get_transaction("0x...")

# Check if it's a Seismic transaction
if hasattr(tx, 'type') and tx.type == SEISMIC_TX_TYPE:
    print("This is a Seismic transaction")
else:
    print("This is a standard Ethereum transaction")
```

### Transaction Type in Logs

```python
from seismic_web3 import SEISMIC_TX_TYPE

print(f"Monitoring for Seismic transactions (type {hex(SEISMIC_TX_TYPE)})...")

# Monitor transactions
for tx in get_recent_transactions():
    if tx.type == SEISMIC_TX_TYPE:
        print(f"Found Seismic transaction: {tx.hash.hex()}")
```

### Transaction Type Filtering

```python
from seismic_web3 import SEISMIC_TX_TYPE, SEISMIC_TESTNET

async def get_seismic_transactions(block_number: int):
    """Get all Seismic transactions from a block."""
    w3 = SEISMIC_TESTNET.public_client()
    block = w3.eth.get_block(block_number, full_transactions=True)

    seismic_txs = [
        tx for tx in block.transactions
        if hasattr(tx, 'type') and tx.type == SEISMIC_TX_TYPE
    ]

    return seismic_txs
```

## EIP-2718 Context

EIP-2718 introduced typed transaction envelopes to Ethereum. Transaction types are identified by a single byte prefix:

| Type | Description |
|------|-------------|
| `0x00` | Legacy transactions (pre-EIP-2718) |
| `0x01` | EIP-2930 (Access lists) |
| `0x02` | EIP-1559 (Dynamic fee) |
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

## Related Constants

Other protocol constants available in the SDK:

```python
from seismic_web3 import (
    SEISMIC_TX_TYPE,              # 0x4A (74)
    SEISMIC_TESTNET_CHAIN_ID,     # 5124
    SANVIL_CHAIN_ID,              # 31337
    TYPED_DATA_MESSAGE_VERSION,   # 2
)
```

## See Also

- [UnsignedSeismicTx](../api-reference/transaction-types/unsigned-seismic-tx.md) - Seismic transaction structure
- [sign_seismic_tx_eip712](../api-reference/eip712/sign-seismic-tx-eip712.md) - Transaction signing
- [ChainConfig](chain-config.md) - Chain configuration with chain IDs
- [Shielded Write Guide](../shielded-write.md) - Guide to sending Seismic transactions
