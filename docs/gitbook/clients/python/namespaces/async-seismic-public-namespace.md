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
    def explorer_url(self, *, item: ExplorerItem | None = None, id: str | int | HexBytes | None = None, tab: str | None = None) -> str | None: ...
    def tx_explorer_url(self, tx_hash: str | HexBytes, *, tab: TxExplorerTab | None = None) -> str | None: ...
    def address_explorer_url(self, address: str, *, tab: AddressExplorerTab | None = None) -> str | None: ...
    def block_explorer_url(self, block_number: int | str, *, tab: BlockExplorerTab | None = None) -> str | None: ...
    def token_explorer_url(self, address: str, *, tab: TokenExplorerTab | None = None) -> str | None: ...
```

## Methods

| Method | Returns | Description |
|--------|---------|-------------|
| [`get_tee_public_key`](methods/get-tee-public-key.md) | `CompressedPublicKey` | Fetch the TEE's compressed secp256k1 public key |
| [`get_deposit_root`](methods/get-deposit-root.md) | `bytes` | Read the deposit Merkle root (32 bytes) |
| [`get_deposit_count`](methods/get-deposit-count.md) | `int` | Read the total validator deposit count |
| `contract` | `AsyncPublicContract` | Create an async read-only contract wrapper (`.tread` only) |
| `tx_explorer_url` | `str \| None` | Generate explorer URL for a transaction |
| `address_explorer_url` | `str \| None` | Generate explorer URL for an address |
| `block_explorer_url` | `str \| None` | Generate explorer URL for a block |
| `token_explorer_url` | `str \| None` | Generate explorer URL for a token |
| `explorer_url` | `str \| None` | Low-level explorer URL builder |

## Example

```python
from seismic_web3 import SEISMIC_TESTNET

public = await SEISMIC_TESTNET.async_public_client()

tee_key = await public.seismic.get_tee_public_key()
root = await public.seismic.get_deposit_root()
count = await public.seismic.get_deposit_count()

# Explorer URLs
tx_url = public.seismic.tx_explorer_url("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")
addr_url = public.seismic.address_explorer_url("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
```

## Notes

- No private key is required
- `contract()` returns `AsyncPublicContract` with `.tread` only

## See Also

- [SeismicPublicNamespace](seismic-public-namespace.md) — Sync equivalent
- [AsyncSeismicNamespace](async-seismic-namespace.md) — Wallet namespace (extends this with write operations)
- [AsyncPublicContract](../contract/public-contract.md) — Contract wrapper returned by `contract()`
