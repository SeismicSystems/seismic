"""ECDH key exchange and AES key derivation.

Implements the Seismic key derivation pipeline:

1. **ECDH** -- elliptic-curve Diffie-Hellman point multiplication
   (``coincurve`` / libsecp256k1).
2. **Shared key extraction** — non-standard Rust-compatible SHA-256
   hash of ``[version_byte || x_coordinate]``.
3. **HKDF-SHA256** — standard key derivation to produce a 32-byte
   AES-256 key (``cryptography`` / OpenSSL backend).

The full pipeline is exposed as :func:`generate_aes_key`, parameterized by
:class:`AesKeyDomain`.
"""

from __future__ import annotations

import hashlib
from enum import Enum

from coincurve import PublicKey as _CoincurvePublicKey
from cryptography.hazmat.primitives.hashes import SHA256
from cryptography.hazmat.primitives.kdf.hkdf import HKDF

from seismic_web3._types import Bytes32, CompressedPublicKey, PrivateKey


class AesKeyDomain(str, Enum):
    """Domain-separation labels for AES keys derived from an ECDH secret.

    HKDF info labels must match the Rust ``AesKeyDomain`` enum (in
    seismic-crypto); keys from different domains are independent.
    """

    #: Client-to-TEE transaction I/O: encrypted calldata, signed-read requests.
    TX_REQUEST = "tx-request"
    #: TEE-to-client transaction I/O: signed-read return data.
    TX_RESPONSE = "tx-response"
    #: The on-chain ECDH precompile's derivation (consensus-frozen label).
    ECDH_PRECOMPILE = "ecdh-precompile"


_DOMAIN_HKDF_INFO = {
    # Request keeps the original "aes-gcm key" label for backward compat;
    # only the response moved to a new one. See the Rust AesKeyDomain enum.
    AesKeyDomain.TX_REQUEST: b"aes-gcm key",
    AesKeyDomain.TX_RESPONSE: b"seismic/response/aes-256-gcm/v1",
    AesKeyDomain.ECDH_PRECOMPILE: b"aes-gcm key",
}


def shared_secret_point(
    private_key: PrivateKey,
    network_public_key: CompressedPublicKey,
) -> bytes:
    """ECDH point multiplication → 64 bytes (x || y, prefix stripped).

    Uses ``coincurve`` (libsecp256k1) for the elliptic-curve operation.

    Args:
        private_key: 32-byte local private key.
        network_public_key: 33-byte compressed TEE public key.

    Returns:
        64-byte uncompressed shared point (without the ``04`` prefix).
    """
    peer = _CoincurvePublicKey(bytes(network_public_key))
    # multiply returns the full 65-byte uncompressed point (04 || x || y)
    shared = peer.multiply(bytes(private_key))
    uncompressed = shared.format(compressed=False)
    return bytes(uncompressed[1:])  # strip the 04 prefix → 64 bytes


def shared_key_from_point(shared_secret: bytes) -> Bytes32:
    """Non-standard Rust-compatible key extraction from an ECDH point.

    Computes::

        version = (shared_secret[63] & 0x01) | 0x02
        SHA-256([version] || x_coordinate[0:32])

    This matches the Rust ``ecies`` crate's shared-secret derivation.

    Args:
        shared_secret: 64-byte ECDH shared point (x || y).

    Returns:
        32-byte derived shared key.
    """
    version = (shared_secret[63] & 0x01) | 0x02
    digest = hashlib.sha256(bytes([version]) + shared_secret[:32]).digest()
    return Bytes32(digest)


def derive_aes_key(shared_key: Bytes32, domain: AesKeyDomain) -> Bytes32:
    """Derive a domain-separated AES-256 key via HKDF-SHA256.

    Uses the ``cryptography`` library (OpenSSL backend).

    Args:
        shared_key: 32-byte input key material (from :func:`shared_key_from_point`).
        domain: The protocol context the key is for.

    Returns:
        32-byte AES-256 key, independent per domain.
    """
    hkdf = HKDF(
        algorithm=SHA256(),
        length=32,
        salt=None,
        info=_DOMAIN_HKDF_INFO[domain],
    )
    return Bytes32(hkdf.derive(bytes(shared_key)))


def generate_aes_key(
    private_key: PrivateKey,
    network_public_key: CompressedPublicKey,
    domain: AesKeyDomain,
) -> Bytes32:
    """Full pipeline: ECDH → shared_key_from_point → HKDF → AES-256 key.

    Args:
        private_key: 32-byte local (encryption) private key.
        network_public_key: 33-byte compressed TEE public key.
        domain: The protocol context the key is for.

    Returns:
        32-byte AES-256 key, independent per domain.
    """
    point = shared_secret_point(private_key, network_public_key)
    shared = shared_key_from_point(point)
    return derive_aes_key(shared, domain)
