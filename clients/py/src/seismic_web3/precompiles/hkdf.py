"""HKDF precompile (address ``0x68``).

On-chain HKDF-SHA256 key derivation.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from seismic_web3._types import Bytes32
from seismic_web3.precompiles._base import (
    Precompile,
    async_call_precompile,
    calc_linear_gas_cost,
    call_precompile,
)

if TYPE_CHECKING:
    from web3 import AsyncWeb3, Web3

HKDF_ADDRESS = "0x0000000000000000000000000000000000000068"

#: Gas constants shared with :mod:`seismic_web3.precompiles.ecdh`.
SHA256_BASE_GAS = 60
SHA256_PER_WORD = 12
HKDF_EXPAND_COST_GAS = 2 * SHA256_BASE_GAS  # 120
SHARED_SECRET_GAS = 3000


def _hkdf_gas_cost(ikm: bytes) -> int:
    linear = calc_linear_gas_cost(
        bus=32,
        length=len(ikm),
        base=SHARED_SECRET_GAS,
        word=SHA256_PER_WORD,
    )
    return 2 * linear + HKDF_EXPAND_COST_GAS


def _hkdf_encode(ikm: bytes) -> bytes:
    return bytes(ikm)


def _hkdf_decode(result: bytes) -> Bytes32:
    return Bytes32(result[:32])


hkdf_precompile: Precompile[bytes, Bytes32] = Precompile(
    address=HKDF_ADDRESS,
    gas_cost=_hkdf_gas_cost,
    encode_params=_hkdf_encode,
    decode_result=_hkdf_decode,
)


def hkdf(w3: Web3, ikm: bytes) -> Bytes32:
    """On-chain HKDF-SHA256 key derivation (sync).

    Args:
        w3: Sync ``Web3`` instance connected to a Seismic node.
        ikm: Input key material (arbitrary bytes).

    Returns:
        32-byte derived key.
    """
    return call_precompile(w3, hkdf_precompile, ikm)


async def async_hkdf(w3: AsyncWeb3, ikm: bytes) -> Bytes32:
    """On-chain HKDF-SHA256 key derivation (async)."""
    return await async_call_precompile(w3, hkdf_precompile, ikm)
