---
description: Standard EVM transaction fields
icon: list
---

# LegacyFields

Standard EVM transaction fields used in metadata construction.

## Overview

`LegacyFields` is a subset of standard Ethereum transaction parameters used when constructing [`TxSeismicMetadata`](tx-seismic-metadata.md) for AAD (Additional Authenticated Data) in AES-GCM encryption.

## Definition

```python
@dataclass(frozen=True)
class LegacyFields:
    """Standard EVM transaction fields used in metadata construction.

    Attributes:
        chain_id: Numeric chain identifier.
        nonce: Sender's transaction count.
        to: Recipient address, or None for contract creation.
        value: Amount of wei to transfer.
    """
    chain_id: int
    nonce: int
    to: ChecksumAddress | None
    value: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `int` | Numeric chain identifier (e.g., 1 for mainnet) |
| `nonce` | `int` | Sender's transaction count |
| `to` | `ChecksumAddress \| None` | Recipient address, or `None` for contract creation |
| `value` | `int` | Amount of wei to transfer |

## Examples

### Manual Construction

```python
from seismic_web3 import LegacyFields

legacy = LegacyFields(
    chain_id=1,
    nonce=42,
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    value=1_000_000_000_000_000_000,  # 1 ETH in wei
)
```

### Contract Creation

```python
from seismic_web3 import LegacyFields

# For contract deployment, set to=None
legacy = LegacyFields(
    chain_id=1,
    nonce=0,
    to=None,  # Contract creation
    value=0,  # No ETH sent
)
```

### Use in TxSeismicMetadata

```python
from seismic_web3 import TxSeismicMetadata, LegacyFields, SeismicElements

metadata = TxSeismicMetadata(
    sender="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    legacy_fields=LegacyFields(
        chain_id=1,
        nonce=42,
        to="0x1234567890123456789012345678901234567890",
        value=1_000_000_000_000_000_000,
    ),
    seismic_elements=SeismicElements(...),
)
```

### Extract from Wallet Client

```python
# SDK automatically extracts these fields
from seismic_web3 import create_wallet_client, SEISMIC_TESTNET, PrivateKey

w3 = create_wallet_client(
    rpc_url="https://rpc.example.com",
    chain=SEISMIC_TESTNET,
    account=PrivateKey(...),
)

# SDK uses chain_id from config
chain_id = w3.eth.chain_id

# SDK fetches nonce automatically
nonce = w3.eth.get_transaction_count(w3.eth.default_account)
```

## Field Details

### chain_id

The numeric chain identifier prevents cross-chain replay attacks:
- **1** - Ethereum mainnet
- **11155111** - Sepolia testnet
- **Custom values** - For Seismic testnets

### nonce

The sender's transaction count ensures:
- **Ordered execution** - Transactions execute in sequence
- **Replay protection** - Can't reuse old transactions
- **Fetched automatically** - SDK calls `eth_getTransactionCount`

### to

The recipient address:
- **Checksummed address** - EIP-55 format (mixed case)
- **`None`** - For contract creation transactions
- **Must be valid** - Invalid addresses cause transaction failure

### value

Amount of native currency to transfer:
- **In wei** - Smallest unit (1 ETH = 10^18 wei)
- **Can be 0** - For pure function calls
- **Integer only** - No fractional wei

## Properties

- **Immutable** - Cannot be modified after construction (`frozen=True`)
- **Type-safe** - All fields are validated at construction
- **Subset of full transaction** - Excludes gas, gasPrice, data

## Why Called "Legacy"?

These are the **standard** EVM fields that exist in all Ethereum transaction types:
- Present in Legacy (pre-EIP-2718) transactions
- Present in EIP-2930 (Type 1) transactions
- Present in EIP-1559 (Type 2) transactions
- Present in Seismic (Type `0x4a`) transactions

The name emphasizes these fields are **unchanged** from traditional Ethereum transactions, unlike the new [`SeismicElements`](seismic-elements.md) fields.

## Notes

- Used exclusively in [`TxSeismicMetadata`](tx-seismic-metadata.md)
- Part of the AAD (Additional Authenticated Data) for AES-GCM encryption
- Automatically extracted by the SDK from wallet and chain config
- Does not include `gas`, `gasPrice`, or `data` fields (those are in [`UnsignedSeismicTx`](unsigned-seismic-tx.md))

## See Also

- [TxSeismicMetadata](tx-seismic-metadata.md) - Uses LegacyFields
- [SeismicElements](seismic-elements.md) - Seismic-specific fields (contrast)
- [UnsignedSeismicTx](unsigned-seismic-tx.md) - Full transaction structure
