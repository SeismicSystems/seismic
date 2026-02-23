---
description: Async read-only Seismic namespace
icon: eye
---

# AsyncSeismicPublicNamespace

Async read-only namespace attached as `w3.seismic` on async public clients.

## Definition

```python
class AsyncSeismicPublicNamespace:
    async def get_tee_public_key(self) -> CompressedPublicKey: ...
    def contract(self, address: ChecksumAddress, abi: list[dict[str, Any]]) -> AsyncPublicContract: ...
    async def get_deposit_root(self, *, address: str = DEPOSIT_CONTRACT_ADDRESS) -> bytes: ...
    async def get_deposit_count(self, *, address: str = DEPOSIT_CONTRACT_ADDRESS) -> int: ...
```

## Example

```python
from seismic_web3 import SEISMIC_TESTNET

public = await SEISMIC_TESTNET.async_public_client()

tee_key = await public.seismic.get_tee_public_key()
root = await public.seismic.get_deposit_root()
count = await public.seismic.get_deposit_count()
```

## Notes

- No private key is required.
- `contract()` returns `AsyncPublicContract` with `.tread` only.
