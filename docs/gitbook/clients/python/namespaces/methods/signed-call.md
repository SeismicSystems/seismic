---
description: Execute signed encrypted eth_call
icon: search
---

# signed_call

Execute a signed read with encrypted calldata.

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

- Encrypts request calldata.
- Signs an unsigned Seismic transaction payload.
- Calls `eth_call` with raw signed bytes.
- Decrypts result before returning.
- Empty RPC result (`"0x"`) returns `HexBytes(b"")`.

## Example

```python
raw = w3.seismic.signed_call(to="0xTarget", data=calldata)
if raw:
    print(raw.hex())
```
