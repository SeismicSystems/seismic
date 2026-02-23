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

    def contract(..., eip712: bool = False) -> AsyncShieldedContract: ...
    async def send_shielded_transaction(..., eip712: bool = False) -> HexBytes: ...
    async def signed_call(..., gas: int = 30_000_000, eip712: bool = False) -> HexBytes: ...
    async def debug_send_shielded_transaction(..., eip712: bool = False) -> DebugWriteResult: ...
    async def deposit(..., address: str = DEPOSIT_CONTRACT_ADDRESS) -> HexBytes: ...
```

## Example

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey, SRC20_ABI

pk = PrivateKey.from_hex_str("0x...")
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

token = w3.seismic.contract("0xTokenAddress", SRC20_ABI)

tx_hash = await token.write.transfer("0xRecipient", 100)
raw = await token.read.balanceOf()
```

## Notes

- Includes every async public namespace method.
- `signed_call()` always returns `HexBytes`; empty responses are `HexBytes(b"")`.
- `deposit()` validates byte lengths and raises `ValueError` on mismatch.
