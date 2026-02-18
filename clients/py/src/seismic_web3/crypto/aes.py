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
