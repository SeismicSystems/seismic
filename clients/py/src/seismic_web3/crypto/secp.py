"""secp256k1 key utilities.

Provides public key compression and private-key-to-public-key derivation
using ``coincurve`` (libsecp256k1 bindings).
"""

from __future__ import annotations

from coincurve import PublicKey as _CoincurvePublicKey

from seismic_web3._types import CompressedPublicKey, PrivateKey


def compress_public_key(uncompressed_key: bytes) -> CompressedPublicKey:
    """Compress a 65-byte uncompressed secp256k1 public key to 33 bytes.

    Args:
        uncompressed_key: 65-byte uncompressed public key
            (``04`` prefix + 32-byte x + 32-byte y).

    Returns:
        33-byte compressed public key (``02`` or ``03`` prefix).

    Raises:
        ValueError: If the input is not a valid 65-byte public key.
    """
    if len(uncompressed_key) != 65:
        raise ValueError(
            f"Expected 65-byte uncompressed key, got {len(uncompressed_key)}"
        )
    pt = _CoincurvePublicKey(uncompressed_key)
    return CompressedPublicKey(pt.format(compressed=True))


def private_key_to_compressed_public_key(
    private_key: PrivateKey,
) -> CompressedPublicKey:
    """Derive the compressed public key from a secp256k1 private key.

    Args:
        private_key: 32-byte private key.

    Returns:
        33-byte compressed public key.
    """
    pt = _CoincurvePublicKey.from_secret(bytes(private_key))
    return CompressedPublicKey(pt.format(compressed=True))
