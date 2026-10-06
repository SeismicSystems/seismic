"""Public, signed gas-payment choices for Seismic transactions."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from eth_utils import is_address, to_checksum_address

_ZERO_ADDRESS = "0x0000000000000000000000000000000000000000"


@dataclass(frozen=True)
class GasPayment:
    """Immutable fee selection; explicit choices never fall back.

    Use ``auto()``, ``native()`` or ``token(address)``. Request omission resolves
    to Auto before signing. This is public metadata, not an encryption option.
    """

    kind: Literal["auto", "native", "token"] = "auto"
    token_address: str | None = None

    def __post_init__(self) -> None:
        if self.kind in ("auto", "native"):
            if self.token_address is not None:
                raise ValueError("Auto/Native gas payment cannot carry a token")
            return
        address = self.token_address
        if (
            self.kind != "token"
            or not isinstance(address, str)
            or not address.startswith("0x")
            or len(address) != 42
            or not is_address(address)
            or address.lower() == _ZERO_ADDRESS
        ):
            raise ValueError("Token gas payment requires a nonzero 20-byte address")
        object.__setattr__(self, "token_address", to_checksum_address(address))

    @classmethod
    def auto(cls) -> GasPayment:
        """Native first, then eligible registered tokens in insertion order."""
        return cls("auto")

    @classmethod
    def native(cls) -> GasPayment:
        """Native funds only, without token fallback."""
        return cls("native")

    @classmethod
    def token(cls, address: str) -> GasPayment:
        """Exactly this registered token, without fallback."""
        return cls("token", address)

    def rlp_parts(self) -> list[bytes]:
        """Canonical nested RLP field, with empty token for Auto/Native."""
        if self.kind == "auto":
            return [b"", b""]
        if self.kind == "native":
            return [b"\x01", b""]
        return [b"\x02", bytes.fromhex(str(self.token_address)[2:])]

    def typed_data(self) -> dict[str, int | str]:
        """EIP-712 kind/address pair; Auto/Native use a zero-address sentinel."""
        return {
            "kind": {"auto": 0, "native": 1, "token": 2}[self.kind],
            "token": self.token_address or _ZERO_ADDRESS,
        }

    def to_json(self) -> dict[str, str]:
        """Tagged JSON representation matching the Rust RPC format."""
        if self.kind == "token":
            return {"type": "token", "token": str(self.token_address)}
        return {"type": self.kind}


def resolve_gas_payment(payment: GasPayment | None = None) -> GasPayment:
    """Resolve a construction default without accepting unsigned wire omissions."""
    if payment is None:
        return GasPayment.auto()
    if not isinstance(payment, GasPayment):
        raise TypeError("gas_payment must be a GasPayment value")
    return payment


def require_auto_payment(payment: GasPayment | None = None) -> None:
    """Reject explicit fee selection on standard Ethereum transaction paths."""
    if resolve_gas_payment(payment).kind != "auto":
        raise ValueError(
            "Non-Auto gas_payment requires a Seismic transaction; use swrite or "
            "send_shielded_transaction",
        )
