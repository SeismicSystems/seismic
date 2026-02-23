---
description: Async wallet Seismic namespace
icon: lock
---

# AsyncSeismicNamespace

`AsyncSeismicNamespace` extends `AsyncSeismicPublicNamespace` with private-key operations.

## Definition

```python
class AsyncSeismicNamespace(AsyncSeismicPublicNamespace):
    encryption: EncryptionState

    def contract(
        self,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
        eip712: bool = False,
    ) -> AsyncShieldedContract: ...

    async def send_shielded_transaction(..., eip712: bool = False) -> HexBytes: ...
    async def signed_call(..., gas: int = 30_000_000, eip712: bool = False) -> HexBytes: ...
    async def debug_send_shielded_transaction(..., eip712: bool = False) -> DebugWriteResult: ...

    async def deposit(
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

- Includes all async public namespace methods.
- `signed_call()` returns `HexBytes`; empty responses are `HexBytes(b"")`.
- `deposit()` validates byte lengths and raises `ValueError` on mismatch.
