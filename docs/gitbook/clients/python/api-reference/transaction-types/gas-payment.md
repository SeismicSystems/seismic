---
description: Immutable per-transaction gas-payment selector
icon: gas-pump
---

# GasPayment

```python
from seismic_web3 import GasPayment

GasPayment.auto()                       # native first, then eligible registry tokens
GasPayment.native()                     # native only, no fallback
GasPayment.token("0xYourGasTokenAddress") # exactly this registered token, no fallback
```

Replace the placeholder with a valid nonzero 20-byte token address. Helpers validate and checksum addresses. Values are frozen: an invalid kind, token attached to Auto/Native, absent/zero Token address, or unknown constructor keyword is rejected.

## Request option

`gas_payment` is a keyword-only, **top-level** option, separate from `security` and encryption metadata. Omission or `None` resolves to `GasPayment.auto()` before signing. The wire selector is mandatory even when the caller omits the option.

```python
from seismic_web3 import GasPayment

payment = GasPayment.token("0xYourGasTokenAddress")

tx_hash = w3.seismic.send_shielded_transaction(
    to="0xYourContractAddress",
    data=calldata,
    gas_payment=payment,
)

tx_hash = token.swrite.transfer(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    1000,
    gas_payment=payment,
)
```

The token used for fees can differ from the contract being called. For a proxy token, select its registered proxy address, not its implementation.

Both sync and async APIs support this option: sends, signed calls, debug sends, contract `.swrite`/`.sread`/`.dwrite`, and smart `.write`/`.read` when the ABI selects the shielded route. Signed estimation twins use the same selector as the final transaction. Debug views show the resolved selection.

Smart routing is unchanged. If `.write`/`.read` selects the transparent path, a non-Auto selector raises an error; `.twrite` likewise rejects it rather than silently dropping it. Standard Ethereum transactions execute with Auto and cannot authenticate explicit Native/Token choices. Force `.swrite`/`.sread` when necessary.

## Transaction lookup responses

The existing `w3.eth.get_transaction`, `get_transaction_by_block`, and `get_block(..., full_transactions=True)` APIs preserve the public `gasPayment` field. Both wallet and public factories still return normal Web3/AsyncWeb3 clients; no replacement `eth` module or response conversion is introduced.

Upstream `web3.types.TxData` does not declare Seismic fields. Use the exported `is_seismic_transaction` type guard to narrow an existing Web3 response to `SeismicTransactionResponse`:

```python
from seismic_web3 import GasPaymentResponse, is_seismic_transaction

transaction = w3.eth.get_transaction(tx_hash)
if is_seismic_transaction(transaction):
    payment: GasPaymentResponse = transaction["gasPayment"]
    if payment["type"] == "token":
        print(payment["token"])

block = w3.eth.get_block(1, full_transactions=True)
for transaction in block["transactions"]:
    if is_seismic_transaction(transaction):
        print(transaction["gasPayment"])
```

With AsyncWeb3, await the same lookup methods before applying the guard. It supports hash, block-hash/index, block-number/index, and full-block transaction results. Ordinary Ethereum transactions and hash-only block entries do not narrow. Missing/malformed selectors do not default to Auto; the guard returns `False` without changing the response.

`SeismicTransactionResponse` is a `TypedDict` extending `TxData`, not a signing dataclass. `GasPaymentResponse` is a discriminated union of mappings:

```python
{"type": "auto"}
{"type": "native"}
{"type": "token", "token": "0xYourGasTokenAddress"}
```

The runtime response and nested payment remain web3.py `AttributeDict` mappings. Access typed fields with dictionary indexing. Token addresses retain the node's string/casing; there is no checksum normalization. Standard Ethereum fields keep web3.py formatting. Additional `encryptionNonce`, `messageVersion`, and `expiresAtBlock` quantities retain their RPC hex strings; `encryptionPubkey` and `recentBlockHash` remain strings, and `signedRead` remains a boolean. The public-key string may omit `0x` according to node serde.

The selector is the **signed preference**, not which asset Auto ultimately used. No calldata decryption, raw transaction decoder, or conversion to the request `GasPayment` dataclass is performed. The guard checks response shape, not transaction validity or consensus rules.

## Fee and privacy semantics

The selector is public signed metadata, not encrypted calldata and not AEAD/AAD. `value` and `gas_price` remain denominated in native wei. With the registry's fixed conversion rate, token fees round up to `ceil(native_wei_fee / 10**(18 - decimals))`. Registration alone does not fund an eligible token balance. Explicit Native and Token never fall back.

Signed reads and estimates carry the selector but do not charge canonical state or consume the account nonce.

## Wire compatibility

The new SDK writes `[kind, token]` immediately after `gasLimit` in raw RLP and a nested `GasPayment(uint8 kind,address token)` in EIP-712. Auto/Native use empty token bytes in RLP and the zero address in typed data. The encryption nonce remains twelve bytes for encryption/AAD, but is a minimal U96 integer in raw RLP.

Use this SDK with a node supporting the mandatory new field. Existing application calls can remain unchanged after upgrading the SDK: omission defaults to Auto. Old already-signed messages without the selector are not accepted by new-format nodes. No old-wire compatibility mode or public custom raw decoder is added. Standard Ethereum bytes and the SDK's existing raw/typed signing defaults are unchanged.
