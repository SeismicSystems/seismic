"""Deposit contract ABI and helpers for Seismic validator staking.

The deposit contract is an Eth2-style validator deposit contract that
manages a Merkle tree of deposits.  Validators deposit ETH (minimum
1 ETH, typically 32 ETH) along with their public keys, signatures,
and withdrawal credentials.

This ABI matches the ``IDepositContract`` interface defined in
``contracts/src/seismic-std-lib/DepositContract.sol``.
"""

from __future__ import annotations

import hashlib
from typing import Any

# ---------------------------------------------------------------------------
# Genesis address — must match seismic-viem DEPOSIT_CONTRACT_ADDRESS
# ---------------------------------------------------------------------------

DEPOSIT_CONTRACT_ADDRESS: str = "0x00000000219ab540356cBB839Cbe05303d7705Fa"

# ---------------------------------------------------------------------------
# ABI — 4 functions + 1 event
# ---------------------------------------------------------------------------

DEPOSIT_CONTRACT_ABI: list[dict[str, Any]] = [
    {
        "type": "function",
        "name": "deposit",
        "inputs": [
            {"name": "node_pubkey", "type": "bytes", "internalType": "bytes"},
            {"name": "consensus_pubkey", "type": "bytes", "internalType": "bytes"},
            {
                "name": "withdrawal_credentials",
                "type": "bytes",
                "internalType": "bytes",
            },
            {"name": "node_signature", "type": "bytes", "internalType": "bytes"},
            {"name": "consensus_signature", "type": "bytes", "internalType": "bytes"},
            {
                "name": "deposit_data_root",
                "type": "bytes32",
                "internalType": "bytes32",
            },
        ],
        "outputs": [],
        "stateMutability": "payable",
    },
    {
        "type": "function",
        "name": "get_deposit_count",
        "inputs": [],
        "outputs": [{"name": "", "type": "bytes", "internalType": "bytes"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "get_deposit_root",
        "inputs": [],
        "outputs": [{"name": "", "type": "bytes32", "internalType": "bytes32"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "supportsInterface",
        "inputs": [
            {"name": "interfaceId", "type": "bytes4", "internalType": "bytes4"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "pure",
    },
    {
        "type": "event",
        "name": "DepositEvent",
        "inputs": [
            {
                "name": "node_pubkey",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "consensus_pubkey",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "withdrawal_credentials",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "amount",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "node_signature",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "consensus_signature",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "index",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
        ],
        "anonymous": False,
    },
]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _sha256(data: bytes) -> bytes:
    """SHA-256 hash, returning raw 32 bytes."""
    return hashlib.sha256(data).digest()


def _to_little_endian_64(value: int) -> bytes:
    """Encode a 64-bit unsigned integer as 8 bytes, little-endian."""
    return value.to_bytes(8, byteorder="little")


def _check_bytes(name: str, value: bytes, expected: int) -> None:
    """Raise ``ValueError`` if *value* is not exactly *expected* bytes."""
    if len(value) != expected:
        msg = f"{name} must be {expected} bytes, got {len(value)}"
        raise ValueError(msg)


def make_withdrawal_credentials(address: str) -> bytes:
    """Build 32-byte ETH1 withdrawal credentials from an Ethereum address.

    Format: ``0x01`` + 11 zero bytes + 20-byte address.

    Args:
        address: Hex-encoded Ethereum address (with or without ``0x`` prefix).

    Returns:
        32-byte withdrawal credentials.

    Raises:
        ValueError: If the address is not exactly 20 bytes.
    """
    addr_bytes = bytes.fromhex(address.replace("0x", "").replace("0X", ""))
    if len(addr_bytes) != 20:
        msg = f"address must be 20 bytes, got {len(addr_bytes)}"
        raise ValueError(msg)
    return b"\x01" + b"\x00" * 11 + addr_bytes


def compute_deposit_data_root(
    *,
    node_pubkey: bytes,
    consensus_pubkey: bytes,
    withdrawal_credentials: bytes,
    node_signature: bytes,
    consensus_signature: bytes,
    amount_gwei: int,
) -> bytes:
    """Compute the deposit data root hash (SHA-256 SSZ hash tree root).

    This mirrors the on-chain verification logic in ``DepositContract.sol``.
    The returned value must be passed as the ``deposit_data_root`` parameter
    when calling ``deposit()``.

    Args:
        node_pubkey: 32-byte ED25519 public key.
        consensus_pubkey: 48-byte BLS12-381 public key.
        withdrawal_credentials: 32-byte withdrawal credentials
            (use :func:`make_withdrawal_credentials`).
        node_signature: 64-byte ED25519 signature.
        consensus_signature: 96-byte BLS12-381 signature.
        amount_gwei: Deposit amount in gwei (e.g. ``32_000_000_000`` for 32 ETH).

    Returns:
        32-byte deposit data root hash.

    Raises:
        ValueError: If any argument has the wrong byte length.
    """
    _check_bytes("node_pubkey", node_pubkey, 32)
    _check_bytes("consensus_pubkey", consensus_pubkey, 48)
    _check_bytes("withdrawal_credentials", withdrawal_credentials, 32)
    _check_bytes("node_signature", node_signature, 64)
    _check_bytes("consensus_signature", consensus_signature, 96)

    amount = _to_little_endian_64(amount_gwei)

    consensus_pubkey_hash = _sha256(consensus_pubkey + b"\x00" * 16)
    pubkey_root = _sha256(node_pubkey + consensus_pubkey_hash)

    node_signature_hash = _sha256(node_signature)
    consensus_signature_hash = _sha256(
        _sha256(consensus_signature[:64])
        + _sha256(consensus_signature[64:] + b"\x00" * 32)
    )
    signature_root = _sha256(node_signature_hash + consensus_signature_hash)

    return _sha256(
        _sha256(pubkey_root + withdrawal_credentials)
        + _sha256(amount + b"\x00" * 24 + signature_root)
    )
