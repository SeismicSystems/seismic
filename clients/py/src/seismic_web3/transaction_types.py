"""Seismic transaction data structures.

Typed, immutable dataclasses representing the Seismic-specific fields
and structures used to build, sign, and serialize ``TxSeismic``
(type ``0x4a``) transactions.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from hexbytes import HexBytes

    from seismic_web3._types import Bytes32, CompressedPublicKey, EncryptionNonce


# ---------------------------------------------------------------------------
# Signature
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Signature:
    """ECDSA signature components (all-or-nothing).

    Attributes:
        v: Recovery identifier (0 or 1, per EIP-155 / y-parity).
        r: First 32-byte integer of the signature.
        s: Second 32-byte integer of the signature.
    """

    v: int
    r: int
    s: int


# ---------------------------------------------------------------------------
# Seismic elements
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class SeismicElements:
    """Seismic-specific fields appended to a transaction.

    These fields are required for every ``TxSeismic`` and carry the
    encryption parameters plus block-expiry metadata.

    Attributes:
        encryption_pubkey: Compressed secp256k1 public key used for
            ECDH-derived calldata encryption.
        encryption_nonce: 12-byte AES-GCM nonce.
        message_version: Signing mode — ``0`` for raw, ``2`` for EIP-712.
        recent_block_hash: Hash of a recent block (freshness proof).
        expires_at_block: Block number after which the tx is invalid.
        signed_read: ``True`` for signed ``eth_call`` reads.
    """

    encryption_pubkey: CompressedPublicKey
    encryption_nonce: EncryptionNonce
    message_version: int
    recent_block_hash: Bytes32
    expires_at_block: int
    signed_read: bool


# ---------------------------------------------------------------------------
# Legacy EVM fields (subset used for metadata / AAD)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class LegacyFields:
    """Standard EVM transaction fields used in metadata construction.

    Attributes:
        chain_id: Numeric chain identifier.
        nonce: Sender's transaction count.
        to: Recipient address, or ``None`` for contract creation.
        value: Amount of wei to transfer.
    """

    chain_id: int
    nonce: int
    to: ChecksumAddress | None
    value: int


# ---------------------------------------------------------------------------
# Transaction metadata (used as AAD context)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class TxSeismicMetadata:
    """Complete metadata for a Seismic transaction.

    Used to construct the Additional Authenticated Data (AAD) for
    AES-GCM encryption, ensuring the ciphertext is bound to the
    full transaction context.

    Attributes:
        sender: Checksummed sender address.
        legacy_fields: Standard EVM fields (chain ID, nonce, to, value).
        seismic_elements: Seismic-specific encryption and expiry fields.
    """

    sender: ChecksumAddress
    legacy_fields: LegacyFields
    seismic_elements: SeismicElements


# ---------------------------------------------------------------------------
# Unsigned Seismic transaction
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class UnsignedSeismicTx:
    """All fields of a ``TxSeismic`` before signing.

    The ``data`` field contains **encrypted** calldata (ciphertext).
    Use the serialization functions in ``transaction.serialize`` to
    RLP-encode this for hashing and signing.

    Attributes:
        chain_id: Numeric chain identifier.
        nonce: Sender's transaction count.
        gas_price: Gas price in wei.
        gas: Gas limit.
        to: Recipient address, or ``None`` for contract creation.
        value: Amount of wei to transfer.
        data: Encrypted calldata (ciphertext).
        seismic: Seismic-specific encryption and expiry fields.
    """

    chain_id: int
    nonce: int
    gas_price: int
    gas: int
    to: ChecksumAddress | None
    value: int
    data: HexBytes
    seismic: SeismicElements


# ---------------------------------------------------------------------------
# Security parameters (user-facing overrides)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class SeismicSecurityParams:
    """Optional security parameters for shielded transactions.

    All fields default to ``None``, meaning the SDK will use sensible
    defaults (e.g. fetch latest block, generate a random nonce, use a
    100-block expiry window).

    Attributes:
        blocks_window: Number of blocks before the tx expires
            (default ``100`` when ``None``).
        encryption_nonce: Explicit AES-GCM nonce (random if ``None``).
        recent_block_hash: Explicit block hash (latest if ``None``).
        expires_at_block: Explicit expiry block (computed if ``None``).
    """

    blocks_window: int | None = None
    encryption_nonce: EncryptionNonce | None = None
    recent_block_hash: Bytes32 | None = None
    expires_at_block: int | None = None
