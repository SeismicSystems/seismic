---
description: Sync read-only Seismic namespace
icon: eye
---

# SeismicPublicNamespace

Sync read-only namespace attached as `w3.seismic` on public clients.

## Definition

```python
class SeismicPublicNamespace:
    def get_tee_public_key(self) -> CompressedPublicKey: ...
    def contract(self, address: ChecksumAddress, abi: list[dict[str, Any]]) -> PublicContract: ...
    def get_deposit_root(self, *, address: str = DEPOSIT_CONTRACT_ADDRESS) -> bytes: ...
    def get_deposit_count(self, *, address: str = DEPOSIT_CONTRACT_ADDRESS) -> int: ...
```

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| [`get_tee_public_key`](methods/get-tee-public-key.md) | `CompressedPublicKey` | Fetch the TEE's compressed secp256k1 public key |
| [`get_deposit_root`](methods/get-deposit-root.md) | `bytes` | Read the deposit Merkle root (32 bytes) |
| [`get_deposit_count`](methods/get-deposit-count.md) | `int` | Read the total validator deposit count |
| `contract` | `PublicContract` | Create a read-only contract wrapper (`.tread` only) |

## Example

```python
from seismic_web3 import SEISMIC_TESTNET

public = SEISMIC_TESTNET.public_client()

tee_key = public.seismic.get_tee_public_key()
root = public.seismic.get_deposit_root()
count = public.seismic.get_deposit_count()
```

## Notes

- No private key is required
- `contract()` returns `PublicContract` with `.tread` only
- `get_deposit_count()` decodes an 8-byte little-endian value from contract return data

## See Also

- [AsyncSeismicPublicNamespace](async-seismic-public-namespace.md) — Async equivalent
- [SeismicNamespace](seismic-namespace.md) — Wallet namespace (extends this with write operations)
- [PublicContract](../contract/public-contract.md) — Contract wrapper returned by `contract()`
