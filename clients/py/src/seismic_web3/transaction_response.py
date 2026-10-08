"""Typed views of existing web3.py lookup responses, without response conversion.

Use :func:`is_seismic_transaction` to narrow hash/index lookups or full-block
transactions. The payment is a tagged mapping, not the signing ``GasPayment``
dataclass. Unknown Seismic fields retain their JSON-RPC representation because
web3.py only formats the standard Ethereum fields.
"""

from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Literal, TypedDict, TypeGuard, cast

from eth_utils import is_address
from web3.types import TxData

from seismic_web3.chains import SEISMIC_TX_TYPE

if TYPE_CHECKING:
    from hexbytes import HexBytes


class AutoGasPaymentResponse(TypedDict):
    type: Literal["auto"]


class NativeGasPaymentResponse(TypedDict):
    type: Literal["native"]


class TokenGasPaymentResponse(TypedDict):
    type: Literal["token"]
    token: str


GasPaymentResponse = (
    AutoGasPaymentResponse | NativeGasPaymentResponse | TokenGasPaymentResponse
)


class SeismicTransactionResponse(TxData):
    """A web3.py-formatted Seismic transaction with unchanged RPC metadata.

    Standard fields use ``TxData`` types. The additional quantities are hex
    strings, the public key is a string (node serde may omit ``0x``), and the
    payment token remains the node's address string without checksum conversion.
    """

    gasPayment: GasPaymentResponse
    encryptionPubkey: str
    encryptionNonce: str
    messageVersion: str
    recentBlockHash: str
    expiresAtBlock: str
    signedRead: bool


def _is_gas_payment_response(payment: object) -> bool:
    if not isinstance(payment, Mapping):
        return False
    fields = cast("Mapping[object, object]", payment)
    kind = fields.get("type")
    if kind in ("auto", "native"):
        return set(fields) == {"type"}
    token = fields.get("token")
    return (
        kind == "token"
        and set(fields) == {"type", "token"}
        and isinstance(token, str)
        and is_address(token)
        and token.lower() != "0x" + "00" * 20
    )


def is_seismic_transaction(
    transaction: TxData | HexBytes,
) -> TypeGuard[SeismicTransactionResponse]:
    """Narrow an existing web3.py response, leaving it and its payment untouched.

    Works for sync/async hash and block/index lookups and full-block entries;
    hash-only block entries and ordinary Ethereum transactions return ``False``.
    A missing/malformed mandatory selector is not silently defaulted to Auto.
    This is a response-shape check, not transaction or consensus validation.
    """
    if not isinstance(transaction, Mapping):
        return False
    response = cast("Mapping[str, object]", transaction)
    if response.get("type") not in (SEISMIC_TX_TYPE, "0x4a", "0x4A"):
        return False
    if not _is_gas_payment_response(response.get("gasPayment")):
        return False
    for key in (
        "encryptionPubkey",
        "encryptionNonce",
        "messageVersion",
        "recentBlockHash",
        "expiresAtBlock",
    ):
        if not isinstance(response.get(key), str):
            return False
    return isinstance(response.get("signedRead"), bool)
