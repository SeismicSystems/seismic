"""Base precompile framework.

Defines the :class:`Precompile` descriptor and the shared
:func:`call_precompile` / :func:`async_call_precompile` callers
that handle gas estimation, encoding, RPC call, and decoding.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import TYPE_CHECKING, Generic, TypeVar

from hexbytes import HexBytes
from web3.types import RPCEndpoint, RPCResponse

if TYPE_CHECKING:
    from collections.abc import Callable

    from web3 import AsyncWeb3, Web3

P = TypeVar("P")
R = TypeVar("R")

#: Every ``eth_call`` costs at least this much gas.
BASE_TX_GAS_COST = 21_000


def calldata_gas_cost(data: bytes) -> int:
    """EVM calldata gas: 4 per zero byte + 16 per non-zero byte.

    Equivalent to the viem formula:
    ``4 * len(data) + 12 * nonZeroCount``.
    """
    non_zero = sum(1 for b in data if b != 0)
    return 4 * len(data) + 12 * non_zero


def calc_linear_gas_cost(
    *,
    bus: int,
    length: int,
    base: int,
    word: int,
) -> int:
    """Linear gas cost: ``base + ceil(length / bus) * word``."""
    words = math.ceil(length / bus) if length > 0 else 0
    return words * word + base


def calc_linear_gas_cost_u32(
    *,
    length: int,
    base: int,
    word: int,
) -> int:
    """Linear gas cost with ``bus=32``."""
    return calc_linear_gas_cost(bus=32, length=length, base=base, word=word)


@dataclass(frozen=True)
class Precompile(Generic[P, R]):
    """Descriptor for a Mercury EVM precompile.

    Attributes:
        address: Hex address (e.g. ``"0x…0064"``).
        gas_cost: Compute execution gas for the given args.
        encode_params: Encode Python args to raw calldata bytes.
        decode_result: Decode raw result bytes to Python type.
    """

    address: str
    gas_cost: Callable[[P], int]
    encode_params: Callable[[P], bytes]
    decode_result: Callable[[bytes], R]


def _build_call_params(precompile: Precompile[P, R], args: P) -> dict[str, str]:
    """Encode args into an ``eth_call`` transaction dict.

    Gas is omitted so the node uses its default (block gas limit).
    For cost estimation, use :func:`calldata_gas_cost` and
    :attr:`Precompile.gas_cost` directly.
    """
    data = precompile.encode_params(args)
    return {
        "to": precompile.address,
        "data": HexBytes(data).to_0x_hex(),
    }


def _extract_result(response: RPCResponse) -> bytes:
    """Extract result bytes from an RPC response, raising on errors."""
    if "error" in response:
        err = response["error"]
        msg = err.get("message", err) if isinstance(err, dict) else err
        raise RuntimeError(f"Precompile call failed: {msg}")
    raw: str = str(response.get("result", "0x"))
    if not raw or raw == "0x":
        raise ValueError("No data returned from precompile")
    return bytes(HexBytes(raw))


def call_precompile(
    w3: Web3,
    precompile: Precompile[P, R],
    args: P,
) -> R:
    """Call a precompile via ``eth_call`` (sync).

    Uses ``w3.provider.make_request`` directly to avoid web3.py
    injecting a ``from`` address (Seismic nodes reject unsigned
    calls with a non-zero sender).

    Args:
        w3: Sync ``Web3`` instance connected to a Seismic node.
        precompile: Precompile descriptor.
        args: Precompile-specific input parameters.

    Returns:
        Decoded result.

    Raises:
        ValueError: If the precompile returns empty data.
        RuntimeError: If the RPC returns an error.
    """
    tx = _build_call_params(precompile, args)
    response = w3.provider.make_request(RPCEndpoint("eth_call"), [tx, "latest"])
    return precompile.decode_result(_extract_result(response))


async def async_call_precompile(
    w3: AsyncWeb3,
    precompile: Precompile[P, R],
    args: P,
) -> R:
    """Call a precompile via ``eth_call`` (async).

    Same as :func:`call_precompile` but for ``AsyncWeb3`` instances.
    """
    tx = _build_call_params(precompile, args)
    response = await w3.provider.make_request(RPCEndpoint("eth_call"), [tx, "latest"])
    return precompile.decode_result(_extract_result(response))
