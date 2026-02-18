"""Seismic transaction metadata builder.

Builds ``TxSeismicMetadata`` by fetching chain state (nonce, latest
block) as needed, with both sync and async variants.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from seismic_web3.crypto.nonce import random_encryption_nonce

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import Bytes32, CompressedPublicKey, EncryptionNonce
    from seismic_web3.transaction_types import TxSeismicMetadata

#: Default number of blocks before a transaction expires.
DEFAULT_BLOCKS_WINDOW = 100


@dataclass
class MetadataParams:
    """Parameters for building ``TxSeismicMetadata``.

    Optional fields (``None``) are automatically fetched from the chain
    or generated.

    Attributes:
        sender: Checksummed sender address.
        to: Recipient address (``None`` for contract creation).
        encryption_pubkey: Compressed public key for ECDH.
        value: Wei to transfer (default ``0``).
        nonce: Sender's tx count (fetched if ``None``).
        blocks_window: Blocks until expiry (default ``100``).
        encryption_nonce: AES-GCM nonce (random if ``None``).
        recent_block_hash: Recent block hash (fetched if ``None``).
        expires_at_block: Explicit expiry block (computed if ``None``).
        message_version: ``0`` for raw, ``2`` for EIP-712.
        signed_read: ``True`` for signed ``eth_call`` reads.
    """

    sender: ChecksumAddress
    to: ChecksumAddress | None
    encryption_pubkey: CompressedPublicKey
    value: int = 0
    nonce: int | None = None
    blocks_window: int = DEFAULT_BLOCKS_WINDOW
    encryption_nonce: EncryptionNonce | None = None
    recent_block_hash: Bytes32 | None = None
    expires_at_block: int | None = None
    message_version: int = 0
    signed_read: bool = False


def _assemble_metadata(
    params: MetadataParams,
    chain_id: int,
    nonce: int,
    recent_block_hash: Bytes32,
    expires_at_block: int,
    encryption_nonce: EncryptionNonce,
) -> TxSeismicMetadata:
    """Pure helper: assemble metadata from resolved values."""
    from seismic_web3.transaction_types import (
        LegacyFields,
        SeismicElements,
        TxSeismicMetadata,
    )

    return TxSeismicMetadata(
        sender=params.sender,
        legacy_fields=LegacyFields(
            chain_id=chain_id,
            nonce=nonce,
            to=params.to,
            value=params.value,
        ),
        seismic_elements=SeismicElements(
            encryption_pubkey=params.encryption_pubkey,
            encryption_nonce=encryption_nonce,
            message_version=params.message_version,
            recent_block_hash=recent_block_hash,
            expires_at_block=expires_at_block,
            signed_read=params.signed_read,
        ),
    )


def build_metadata(w3: Web3, params: MetadataParams) -> TxSeismicMetadata:
    """Build ``TxSeismicMetadata``, fetching chain state as needed (sync).

    Resolves ``nonce``, ``recent_block_hash``, and ``expires_at_block``
    from the connected node if not explicitly provided in ``params``.

    Args:
        w3: Sync ``Web3`` instance.
        params: Metadata parameters (some may be ``None``).

    Returns:
        Fully populated ``TxSeismicMetadata``.
    """
    from seismic_web3._types import Bytes32

    chain_id = w3.eth.chain_id
    nonce = (
        params.nonce
        if params.nonce is not None
        else w3.eth.get_transaction_count(params.sender)
    )
    enc_nonce = params.encryption_nonce or random_encryption_nonce()

    if params.recent_block_hash is not None and params.expires_at_block is not None:
        block_hash = params.recent_block_hash
        expires = params.expires_at_block
    else:
        block = w3.eth.get_block("latest")
        block_hash = params.recent_block_hash or Bytes32(block["hash"])
        if params.expires_at_block is not None:
            expires = params.expires_at_block
        else:
            expires = block["number"] + params.blocks_window

    return _assemble_metadata(params, chain_id, nonce, block_hash, expires, enc_nonce)


async def async_build_metadata(
    w3: AsyncWeb3, params: MetadataParams
) -> TxSeismicMetadata:
    """Build ``TxSeismicMetadata``, fetching chain state as needed (async).

    Async variant of :func:`build_metadata`.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        params: Metadata parameters (some may be ``None``).

    Returns:
        Fully populated ``TxSeismicMetadata``.
    """
    from seismic_web3._types import Bytes32

    chain_id = await w3.eth.chain_id
    nonce = (
        params.nonce
        if params.nonce is not None
        else await w3.eth.get_transaction_count(params.sender)
    )
    enc_nonce = params.encryption_nonce or random_encryption_nonce()

    if params.recent_block_hash is not None and params.expires_at_block is not None:
        block_hash = params.recent_block_hash
        expires = params.expires_at_block
    else:
        block = await w3.eth.get_block("latest")
        block_hash = params.recent_block_hash or Bytes32(block["hash"])
        if params.expires_at_block is not None:
            expires = params.expires_at_block
        else:
            expires = block["number"] + params.blocks_window

    return _assemble_metadata(params, chain_id, nonce, block_hash, expires, enc_nonce)
