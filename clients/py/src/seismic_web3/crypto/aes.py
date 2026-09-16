"""AES-256-GCM encryption and decryption.

Uses the ``cryptography`` library's AESGCM implementation
(OpenSSL backend) for authenticated encryption.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from hexbytes import HexBytes

if TYPE_CHECKING:
    from seismic_web3._types import Bytes32, EncryptionNonce


RESPONSE_IV_LENGTH = 12
"""Length of the IV the TEE prepends to a signed-read response."""

RESPONSE_FORMAT_VERSION = 1
"""Wire format version prefixing every signed-read response."""

MIN_RESPONSE_LEN = 1 + RESPONSE_IV_LENGTH + 16
"""Smallest well-formed response: version, IV, and AES-GCM tag over empty plaintext."""


def split_response_iv(response: HexBytes) -> tuple[int, HexBytes, HexBytes]:
    """Split a signed-read response of the form ``version || iv || ciphertext || tag``.

    Args:
        response: Raw response bytes returned by the node.

    Returns:
        A ``(version, iv, body)`` triple.

    Raises:
        ValueError: If the response is too short, or carries an unknown version.
    """
    if len(response) < MIN_RESPONSE_LEN:
        raise ValueError(
            f"signed-read response is {len(response)} bytes, shorter than the "
            f"{MIN_RESPONSE_LEN}-byte minimum envelope",
        )
    version = response[0]
    if version != RESPONSE_FORMAT_VERSION:
        raise ValueError(
            f"unsupported signed-read response format {version}, "
            f"expected {RESPONSE_FORMAT_VERSION}",
        )
    return (
        version,
        HexBytes(bytes(response[1 : 1 + RESPONSE_IV_LENGTH])),
        HexBytes(bytes(response[1 + RESPONSE_IV_LENGTH :])),
    )


class AesGcmCrypto:
    """AES-256-GCM authenticated encryption / decryption.

    Wraps ``cryptography.hazmat.primitives.ciphers.aead.AESGCM``
    with the Seismic SDK's typed byte primitives.

    Args:
        key: 32-byte AES-256 key.

    Raises:
        ValueError: If the key is not exactly 32 bytes.
    """

    def __init__(self, key: Bytes32) -> None:
        self._cipher = AESGCM(bytes(key))

    def encrypt(
        self,
        plaintext: HexBytes,
        nonce: EncryptionNonce,
        aad: bytes | None = None,
    ) -> HexBytes:
        """Encrypt plaintext using AES-256-GCM.

        Returns ciphertext with the 16-byte authentication tag appended.
        Empty plaintext (``b""``) returns empty bytes.

        Args:
            plaintext: Data to encrypt.
            nonce: 12-byte AES-GCM nonce.
            aad: Optional Additional Authenticated Data.

        Returns:
            Ciphertext || 16-byte auth tag.
        """
        if len(plaintext) == 0:
            return HexBytes(b"")
        ct = self._cipher.encrypt(bytes(nonce), bytes(plaintext), aad)
        return HexBytes(ct)

    def decrypt(
        self,
        ciphertext: HexBytes,
        nonce: EncryptionNonce,
        aad: bytes | None = None,
    ) -> HexBytes:
        """Decrypt ciphertext using AES-256-GCM.

        The ciphertext must include the 16-byte authentication tag
        (appended by :meth:`encrypt`).  Empty ciphertext returns
        empty bytes.

        Args:
            ciphertext: Data to decrypt (includes auth tag).
            nonce: 12-byte AES-GCM nonce.
            aad: Optional Additional Authenticated Data (must match
                what was used during encryption).

        Returns:
            Decrypted plaintext.

        Raises:
            cryptography.exceptions.InvalidTag: If authentication fails.
        """
        if len(ciphertext) == 0:
            return HexBytes(b"")
        pt = self._cipher.decrypt(bytes(nonce), bytes(ciphertext), aad)
        return HexBytes(pt)
