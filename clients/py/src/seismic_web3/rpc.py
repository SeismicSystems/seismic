"""Seismic custom JSON-RPC methods.

Standalone sync and async functions wrapping the ``seismic_*`` JSON-RPC
methods exposed by Seismic nodes.  These are thin wrappers that can be
called with any ``Web3`` / ``AsyncWeb3`` instance — no special module
attachment required.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from web3.types import RPCEndpoint

from seismic_web3._types import CompressedPublicKey

if TYPE_CHECKING:
    from web3 import AsyncWeb3, Web3

#: RPC method name for fetching the TEE's secp256k1 public key.
_TEE_PK_METHOD = RPCEndpoint("seismic_getTeePublicKey")


def get_tee_public_key(w3: Web3) -> CompressedPublicKey:
    """Fetch the TEE's compressed secp256k1 public key (sync).

    Calls the ``seismic_getTeePublicKey`` JSON-RPC method on the
    connected Seismic node.

    Args:
        w3: A **sync** ``Web3`` instance connected to a Seismic node.

    Returns:
        The 33-byte compressed public key used for ECDH key exchange.

    Raises:
        ValueError: If the returned key is not a valid 33-byte
            compressed public key.
    """
    response = w3.provider.make_request(_TEE_PK_METHOD, [])
    raw: str = response["result"]
    hex_key = raw if raw.startswith("0x") else f"0x{raw}"
    return CompressedPublicKey(hex_key)


async def async_get_tee_public_key(w3: AsyncWeb3) -> CompressedPublicKey:
    """Fetch the TEE's compressed secp256k1 public key (async).

    Calls the ``seismic_getTeePublicKey`` JSON-RPC method on the
    connected Seismic node.

    Args:
        w3: An **async** ``AsyncWeb3`` instance connected to a Seismic node.

    Returns:
        The 33-byte compressed public key used for ECDH key exchange.

    Raises:
        ValueError: If the returned key is not a valid 33-byte
            compressed public key.
    """
    response = await w3.provider.make_request(_TEE_PK_METHOD, [])
    raw: str = response["result"]
    hex_key = raw if raw.startswith("0x") else f"0x{raw}"
    return CompressedPublicKey(hex_key)
