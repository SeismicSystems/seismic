---
description: Sync wallet Seismic namespace
icon: wallet
---

# SeismicNamespace

`SeismicNamespace` extends `SeismicPublicNamespace` with private-key operations.

## Definition

```python
class SeismicNamespace(SeismicPublicNamespace):
    encryption: EncryptionState

    def contract(..., eip712: bool = False) -> ShieldedContract: ...
    def send_shielded_transaction(..., eip712: bool = False) -> HexBytes: ...
    def signed_call(..., gas: int = 30_000_000, eip712: bool = False) -> HexBytes: ...
    def debug_send_shielded_transaction(..., eip712: bool = False) -> DebugWriteResult: ...
    def deposit(..., address: str = DEPOSIT_CONTRACT_ADDRESS) -> HexBytes: ...
```

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| [`send_shielded_transaction`](methods/send-shielded-transaction.md) | `HexBytes` | Encrypt, sign, and broadcast a shielded transaction |
| [`signed_call`](methods/signed-call.md) | `HexBytes` | Execute a signed read with encrypted calldata |
| [`debug_send_shielded_transaction`](methods/debug-send-shielded-transaction.md) | `DebugWriteResult` | Send shielded transaction and return debug artifacts |
| [`deposit`](methods/deposit.md) | `HexBytes` | Submit a validator deposit (transparent) |
| [`contract`](../contract/shielded-contract.md) | `ShieldedContract` | Create a shielded contract wrapper |
| `get_tee_public_key` | `CompressedPublicKey` | Inherited from [`SeismicPublicNamespace`](seismic-public-namespace.md) |
| `get_deposit_root` | `bytes` | Inherited from [`SeismicPublicNamespace`](seismic-public-namespace.md) |
| `get_deposit_count` | `int` | Inherited from [`SeismicPublicNamespace`](seismic-public-namespace.md) |

## Example

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey, SRC20_ABI

pk = PrivateKey.from_hex_str("0x...")
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xTokenAddress", SRC20_ABI)

tx_hash = token.write.transfer("0xRecipient", 100)
raw = token.read.balanceOf()
```

## Notes

- Includes every public namespace method
- `signed_call()` always returns `HexBytes`; empty responses are `HexBytes(b"")`
- `deposit()` validates byte lengths and raises `ValueError` on mismatch
- Attached as `w3.seismic` by `CHAIN.wallet_client(pk)`

## See Also

- [AsyncSeismicNamespace](async-seismic-namespace.md) — Async equivalent
- [SeismicPublicNamespace](seismic-public-namespace.md) — Read-only base class
- [ShieldedContract](../contract/shielded-contract.md) — Contract wrapper returned by `contract()`
