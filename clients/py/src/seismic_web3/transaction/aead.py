"""Additional Authenticated Data (AAD) encoding for Seismic transactions.

Encodes the 11 metadata fields as RLP for use as AAD in AES-GCM
encryption, matching the Rust ``TxSeismicMetadata::encode_as_aad``
implementation.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import rlp

if TYPE_CHECKING:
    from seismic_web3.transaction_types import TxSeismicMetadata


def _int_to_rlp_bytes(value: int) -> bytes:
    """Encode an integer as minimal big-endian bytes for RLP.

    Zero is encoded as empty bytes (``b""``), matching Rust/viem
    convention where RLP treats zero as the empty string.

    Args:
        value: Non-negative integer.

    Returns:
        Minimal big-endian byte representation.
    """
    if value == 0:
        return b""
    return value.to_bytes((value.bit_length() + 7) // 8, "big")


def _bool_to_rlp_bytes(value: bool) -> bytes:
    """Encode a boolean for RLP: ``True`` -> ``b"\\x01"``, ``False`` -> ``b""``."""
    return b"\x01" if value else b""


def _address_to_bytes(address: str | None) -> bytes:
    """Convert a checksummed address string to 20 raw bytes.

    ``None`` (contract creation) is encoded as empty bytes.
    """
    if address is None:
        return b""
    return bytes.fromhex(address[2:])  # strip 0x prefix


def encode_metadata_as_aad(metadata: TxSeismicMetadata) -> bytes:
    """RLP-encode the 11 metadata fields as Additional Authenticated Data.

    Field order matches the Rust ``TxSeismicMetadata::encode_as_aad``:

    1. sender
    2. chainId
    3. nonce
    4. to
    5. value
    6. encryptionPubkey
    7. encryptionNonce
    8. messageVersion
    9. recentBlockHash
    10. expiresAtBlock
    11. signedRead

    Args:
        metadata: Complete transaction metadata.

    Returns:
        RLP-encoded byte string suitable for use as AEAD ``aad``.
    """
    lf = metadata.legacy_fields
    se = metadata.seismic_elements

    fields: list[bytes] = [
        _address_to_bytes(metadata.sender),
        _int_to_rlp_bytes(lf.chain_id),
        _int_to_rlp_bytes(lf.nonce),
        _address_to_bytes(lf.to),
        _int_to_rlp_bytes(lf.value),
        bytes(se.encryption_pubkey),
        bytes(se.encryption_nonce),
        _int_to_rlp_bytes(se.message_version),
        bytes(se.recent_block_hash),
        _int_to_rlp_bytes(se.expires_at_block),
        _bool_to_rlp_bytes(se.signed_read),
    ]

    return rlp.encode(fields)
