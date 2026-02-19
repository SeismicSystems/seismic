"""Directory contract helpers for viewing key management.

Ports ``seismic-viem/src/actions/src20/directory.ts``.

The Directory genesis contract (``0x1000…0004``) stores per-user
AES-256 encryption keys.  These helpers provide sync and async access
to register, retrieve, and query viewing keys.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from eth_abi import decode as abi_decode
from eth_hash.auto import keccak

from seismic_web3._types import Bytes32
from seismic_web3.abis.directory import DIRECTORY_ABI
from seismic_web3.contract.abi import encode_shielded_calldata
from seismic_web3.transaction.send import (
    async_send_shielded_transaction,
    async_signed_call,
    send_shielded_transaction,
    signed_call,
)

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from hexbytes import HexBytes
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import PrivateKey
    from seismic_web3.client import EncryptionState

_DIRECTORY_CHECKSUM: ChecksumAddress = (  # type: ignore[assignment]
    "0x1000000000000000000000000000000000000004"
)

_ZERO_KEY = b"\x00" * 32


# ---------------------------------------------------------------------------
# Pure helpers
# ---------------------------------------------------------------------------


def compute_key_hash(aes_key: Bytes32) -> bytes:
    """Compute ``keccak256(aes_key)`` for event topic filtering.

    Args:
        aes_key: 32-byte AES-256 viewing key.

    Returns:
        32-byte hash.
    """
    return keccak(bytes(aes_key))


# ---------------------------------------------------------------------------
# Sync helpers
# ---------------------------------------------------------------------------


def get_viewing_key(
    w3: Web3,
    encryption: EncryptionState,
    private_key: PrivateKey,
) -> Bytes32:
    """Fetch the caller's viewing key from the Directory contract (sync).

    Uses a **signed read** so that ``msg.sender`` is authenticated.

    Args:
        w3: Sync ``Web3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.

    Returns:
        32-byte AES viewing key.

    Raises:
        ValueError: If no key is registered for the caller.
    """
    data = encode_shielded_calldata(DIRECTORY_ABI, "getKey", [])
    result = signed_call(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=_DIRECTORY_CHECKSUM,
        data=data,
    )
    if result is None or bytes(result) == b"" or bytes(result)[-32:] == _ZERO_KEY:
        raise ValueError("No viewing key registered in Directory for this address")
    return Bytes32(bytes(result)[-32:])


def register_viewing_key(
    w3: Web3,
    encryption: EncryptionState,
    private_key: PrivateKey,
    key: Bytes32,
) -> HexBytes:
    """Register a viewing key in the Directory contract (sync).

    Uses a **shielded write** (``setKey(suint256)``).

    Args:
        w3: Sync ``Web3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        key: 32-byte AES viewing key to register.

    Returns:
        Transaction hash.
    """
    data = encode_shielded_calldata(
        DIRECTORY_ABI, "setKey", [int.from_bytes(bytes(key), "big")]
    )
    return send_shielded_transaction(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=_DIRECTORY_CHECKSUM,
        data=data,
    )


def check_has_key(w3: Web3, address: ChecksumAddress) -> bool:
    """Check whether an address has a key in the Directory (sync).

    Plain ``eth_call`` — no encryption needed.
    """
    data = encode_shielded_calldata(DIRECTORY_ABI, "checkHasKey", [address])
    raw = w3.eth.call({"to": _DIRECTORY_CHECKSUM, "data": data})
    (has_key,) = abi_decode(["bool"], bytes(raw))
    return bool(has_key)


def get_key_hash(w3: Web3, address: ChecksumAddress) -> bytes:
    """Get the keccak256 hash of an address's key from the Directory (sync).

    Plain ``eth_call`` — no encryption needed.

    Returns:
        32-byte key hash.
    """
    data = encode_shielded_calldata(DIRECTORY_ABI, "keyHash", [address])
    raw = w3.eth.call({"to": _DIRECTORY_CHECKSUM, "data": data})
    (key_hash,) = abi_decode(["bytes32"], bytes(raw))
    return bytes(key_hash)


# ---------------------------------------------------------------------------
# Async helpers
# ---------------------------------------------------------------------------


async def async_get_viewing_key(
    w3: AsyncWeb3,
    encryption: EncryptionState,
    private_key: PrivateKey,
) -> Bytes32:
    """Fetch the caller's viewing key from the Directory contract (async).

    Uses a **signed read** so that ``msg.sender`` is authenticated.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.

    Returns:
        32-byte AES viewing key.

    Raises:
        ValueError: If no key is registered for the caller.
    """
    data = encode_shielded_calldata(DIRECTORY_ABI, "getKey", [])
    result = await async_signed_call(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=_DIRECTORY_CHECKSUM,
        data=data,
    )
    if result is None or bytes(result) == b"" or bytes(result)[-32:] == _ZERO_KEY:
        raise ValueError("No viewing key registered in Directory for this address")
    return Bytes32(bytes(result)[-32:])


async def async_register_viewing_key(
    w3: AsyncWeb3,
    encryption: EncryptionState,
    private_key: PrivateKey,
    key: Bytes32,
) -> HexBytes:
    """Register a viewing key in the Directory contract (async).

    Uses a **shielded write** (``setKey(suint256)``).

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        key: 32-byte AES viewing key to register.

    Returns:
        Transaction hash.
    """
    data = encode_shielded_calldata(
        DIRECTORY_ABI, "setKey", [int.from_bytes(bytes(key), "big")]
    )
    return await async_send_shielded_transaction(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=_DIRECTORY_CHECKSUM,
        data=data,
    )


async def async_check_has_key(w3: AsyncWeb3, address: ChecksumAddress) -> bool:
    """Check whether an address has a key in the Directory (async).

    Plain ``eth_call`` — no encryption needed.
    """
    data = encode_shielded_calldata(DIRECTORY_ABI, "checkHasKey", [address])
    raw = await w3.eth.call({"to": _DIRECTORY_CHECKSUM, "data": data})
    (has_key,) = abi_decode(["bool"], bytes(raw))
    return bool(has_key)


async def async_get_key_hash(w3: AsyncWeb3, address: ChecksumAddress) -> bytes:
    """Get the keccak256 hash of an address's key from the Directory (async).

    Plain ``eth_call`` — no encryption needed.

    Returns:
        32-byte key hash.
    """
    data = encode_shielded_calldata(DIRECTORY_ABI, "keyHash", [address])
    raw = await w3.eth.call({"to": _DIRECTORY_CHECKSUM, "data": data})
    (key_hash,) = abi_decode(["bytes32"], bytes(raw))
    return bytes(key_hash)
