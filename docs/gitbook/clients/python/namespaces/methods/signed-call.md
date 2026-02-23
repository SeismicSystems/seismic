---
description: Execute signed encrypted eth_call
icon: search
---

# signed_call

Execute a signed read using encrypted calldata.

## Signatures

```python
# sync
w3.seismic.signed_call(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int = 30_000_000,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> HexBytes

# async
await w3.seismic.signed_call(...same args...) -> HexBytes
```

## Behavior

- `data` is plaintext calldata; SDK encrypts it.
- SDK signs a synthetic transaction and sends it to `eth_call` as raw tx bytes.
- If the RPC result is empty (`"0x"`), returns `HexBytes(b"")`.
- Otherwise decrypts and returns response bytes.
