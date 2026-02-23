---
description: Sync wallet Seismic namespace
icon: lock
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

- Includes every public namespace method.
- `signed_call()` always returns `HexBytes`; empty responses are `HexBytes(b"")`.
- `deposit()` validates byte lengths and raises `ValueError` on mismatch.
