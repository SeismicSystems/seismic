"""RNG precompile (address ``0x64``).

Generates on-chain random bytes, returned as a Python ``int``.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from seismic_web3.precompiles._base import (
    Precompile,
    async_call_precompile,
    calc_linear_gas_cost_u32,
    call_precompile,
)

if TYPE_CHECKING:
    from web3 import AsyncWeb3, Web3

RNG_ADDRESS = "0x0000000000000000000000000000000000000064"
# Gas schedule of the node's RNG precompile:
# https://github.com/SeismicSystems/seismic-revm/blob/seismic/crates/seismic/src/precompiles/rng/precompile.rs
_RNG_BASE_GAS = 3500
_RNG_PERS_WORD_GAS = 5
_RNG_ROUND_BASE_GAS = 120
_RNG_ROUND_WORD_GAS = 24
# Each HKDF expansion round hashes the 121-byte domain-separation prefix, the
# personalization, the previous 32-byte block and the 1-byte round counter.
_RNG_INFO_PREFIX_LEN = 121
_HKDF_ROUND_OVERHEAD_LEN = 33


@dataclass(frozen=True)
class RngParams:
    """Parameters for on-chain RNG.

    Attributes:
        num_bytes: Number of random bytes to generate (1--32).
        pers: Optional personalization bytes to seed the RNG.
    """

    num_bytes: int
    pers: bytes = b""


def _rng_gas_cost(params: RngParams) -> int:
    init_cost = calc_linear_gas_cost_u32(
        length=len(params.pers),
        base=_RNG_BASE_GAS,
        word=_RNG_PERS_WORD_GAS,
    )
    round_cost = calc_linear_gas_cost_u32(
        length=_RNG_INFO_PREFIX_LEN + len(params.pers) + _HKDF_ROUND_OVERHEAD_LEN,
        base=_RNG_ROUND_BASE_GAS,
        word=_RNG_ROUND_WORD_GAS,
    )
    rounds = -(-params.num_bytes // 32)
    return init_cost + rounds * round_cost


def _rng_encode(params: RngParams) -> bytes:
    if not 1 <= params.num_bytes <= 32:
        raise ValueError(f"num_bytes must be 1-32, got {params.num_bytes}")
    encoded = params.num_bytes.to_bytes(4, "big")
    if params.pers:
        encoded += params.pers
    return encoded


def _rng_decode(result: bytes) -> int:
    # Pad to 32 bytes then interpret as uint256.
    padded = result.rjust(32, b"\x00")
    return int.from_bytes(padded, "big")


rng_precompile: Precompile[RngParams, int] = Precompile(
    address=RNG_ADDRESS,
    gas_cost=_rng_gas_cost,
    encode_params=_rng_encode,
    decode_result=_rng_decode,
)


def rng(w3: Web3, *, num_bytes: int, pers: bytes = b"") -> int:
    """Generate random bytes on-chain (sync).

    Args:
        w3: Sync ``Web3`` instance connected to a Seismic node.
        num_bytes: Number of random bytes (1--32).
        pers: Optional personalization string.

    Returns:
        Random value as a Python integer.
    """
    return call_precompile(w3, rng_precompile, RngParams(num_bytes, pers))


async def async_rng(
    w3: AsyncWeb3,
    *,
    num_bytes: int,
    pers: bytes = b"",
) -> int:
    """Generate random bytes on-chain (async)."""
    return await async_call_precompile(w3, rng_precompile, RngParams(num_bytes, pers))
