---
description: Send encrypted TxSeismic transaction
icon: shield-halved
---

# send_shielded_transaction

Build, encrypt, sign, and broadcast a shielded transaction.

## Signatures

```python
# sync
w3.seismic.send_shielded_transaction(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> HexBytes

# async
await w3.seismic.send_shielded_transaction(...same args...) -> HexBytes
```

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `to` | `ChecksumAddress` | Required | Recipient contract address |
| `data` | `HexBytes` | Required | Plaintext calldata (SDK encrypts it) |
| `value` | `int` | `0` | Wei to transfer |
| `gas` | `int \| None` | `None` | Gas limit (defaults to `30_000_000`) |
| `gas_price` | `int \| None` | `None` | Gas price in wei (fetched from chain if `None`) |
| `security` | [`SeismicSecurityParams`](../../api-reference/transaction-types/seismic-security-params.md) `\| None` | `None` | Override default security parameters |
| `eip712` | `bool` | `False` | Use EIP-712 typed-data signing path |

## Returns

`HexBytes` — transaction hash from `eth_sendRawTransaction`.

## Example

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(SRC20_ABI, "transfer", ["0xRecipient", 100])
tx_hash = w3.seismic.send_shielded_transaction(to="0xTokenAddress", data=data)
```

## What's encrypted

The SDK encrypts the `data` field (function selector + arguments) using AES-GCM with the ECDH-derived key. An observer can see `from`, `to`, `value`, and gas parameters, but **not** which function was called or what arguments were passed.

## Notes

- For contract interactions, prefer `contract.write.functionName(...)` which handles ABI encoding and encryption automatically
- Creates a `TxSeismic` (type `0x4a`) transaction
- SDK auto-fetches nonce via `eth_getTransactionCount`

## See Also

- [debug_send_shielded_transaction](debug-send-shielded-transaction.md) — Same pipeline with debug artifacts
- [signed_call](signed-call.md) — Read-only equivalent (no state change)
- [contract.write](../../contract/namespaces/write.md) — High-level contract write API
- [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) — Security parameter reference
