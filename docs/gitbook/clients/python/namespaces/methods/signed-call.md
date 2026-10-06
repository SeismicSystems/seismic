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
    gas_payment: GasPayment | None = None,
) -> HexBytes

# async
await w3.seismic.signed_call(...same args...) -> HexBytes
```

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `to` | `ChecksumAddress` | Required | Contract address to call |
| `data` | `HexBytes` | Required | Plaintext calldata (SDK encrypts it) |
| `value` | `int` | `0` | Wei to include in the call context |
| `gas` | `int` | `30_000_000` | Gas limit |
| `security` | [`SeismicSecurityParams`](../../api-reference/transaction-types/seismic-security-params.md) `\| None` | `None` | Override default security parameters |
| `eip712` | `bool` | `False` | Use EIP-712 typed-data signing path |
| `gas_payment` | [`GasPayment`](../../api-reference/transaction-types/gas-payment.md) `\| None` | Auto | Public signed fee choice; does not charge canonical state |

## Returns

`HexBytes` — decrypted response bytes. An authenticated empty response returns `HexBytes(b"")`; a bare RPC `"0x"` is rejected as a truncated encryption envelope.

## Example

```python
raw = w3.seismic.signed_call(to="0xTarget", data=calldata)
if raw:
    print(raw.hex())
```

## Notes

- Does **not** modify blockchain state — this is a read-only `eth_call`
- Both request and response are encrypted end-to-end (calldata, function selector, arguments, and the returned data)
- Does not increment your account nonce
- For contract interactions, prefer `contract.read.functionName(...)` which handles ABI encoding/decoding automatically

## See Also

- [send_shielded_transaction](send-shielded-transaction.md) — Write equivalent (modifies state)
- [contract.read](../../contract/namespaces/read.md) — High-level signed read API
- [contract.tread](../../contract/namespaces/tread.md) — Transparent reads (no encryption)
- [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) — Security parameter reference
