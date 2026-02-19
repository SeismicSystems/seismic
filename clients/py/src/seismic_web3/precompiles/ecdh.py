"""ECDH precompile (address ``0x65``).

Performs on-chain elliptic-curve Diffie-Hellman key exchange.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from seismic_web3._types import Bytes32, CompressedPublicKey, PrivateKey
from seismic_web3.precompiles._base import (
    Precompile,
    async_call_precompile,
    call_precompile,
)
from seismic_web3.precompiles.hkdf import HKDF_EXPAND_COST_GAS, SHARED_SECRET_GAS

if TYPE_CHECKING:
    from web3 import AsyncWeb3, Web3

ECDH_ADDRESS = "0x0000000000000000000000000000000000000065"


@dataclass(frozen=True)
class EcdhParams:
    """Parameters for on-chain ECDH.

    Attributes:
        sk: 32-byte secret key.
        pk: 33-byte compressed public key.
    """

    sk: PrivateKey
    pk: CompressedPublicKey


def _ecdh_gas_cost(_params: EcdhParams) -> int:
    return SHARED_SECRET_GAS + HKDF_EXPAND_COST_GAS  # 3120


def _ecdh_encode(params: EcdhParams) -> bytes:
    return bytes(params.sk) + bytes(params.pk)


def _ecdh_decode(result: bytes) -> Bytes32:
    return Bytes32(result[:32])


ecdh_precompile: Precompile[EcdhParams, Bytes32] = Precompile(
    address=ECDH_ADDRESS,
    gas_cost=_ecdh_gas_cost,
    encode_params=_ecdh_encode,
    decode_result=_ecdh_decode,
)


def ecdh(
    w3: Web3,
    *,
    sk: PrivateKey,
    pk: CompressedPublicKey,
) -> Bytes32:
    """On-chain ECDH key exchange (sync).

    Args:
        w3: Sync ``Web3`` instance connected to a Seismic node.
        sk: 32-byte secret key.
        pk: 33-byte compressed public key.

    Returns:
        32-byte shared secret.
    """
    return call_precompile(w3, ecdh_precompile, EcdhParams(sk, pk))


async def async_ecdh(
    w3: AsyncWeb3,
    *,
    sk: PrivateKey,
    pk: CompressedPublicKey,
) -> Bytes32:
    """On-chain ECDH key exchange (async)."""
    return await async_call_precompile(w3, ecdh_precompile, EcdhParams(sk, pk))
