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

    def contract(
        self,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
        eip712: bool = False,
    ) -> ShieldedContract: ...

    def send_shielded_transaction(..., eip712: bool = False) -> HexBytes: ...
    def signed_call(..., gas: int = 30_000_000, eip712: bool = False) -> HexBytes: ...
    def debug_send_shielded_transaction(..., eip712: bool = False) -> DebugWriteResult: ...

    def deposit(
        *,
        node_pubkey: bytes,
        consensus_pubkey: bytes,
        withdrawal_credentials: bytes,
        node_signature: bytes,
        consensus_signature: bytes,
        deposit_data_root: bytes,
        value: int,
        address: str = DEPOSIT_CONTRACT_ADDRESS,
    ) -> HexBytes: ...
```

## Notes

- Includes all public namespace methods.
- `signed_call()` returns `HexBytes`; empty responses are `HexBytes(b"")`.
- `deposit()` validates byte lengths and raises `ValueError` on mismatch.
