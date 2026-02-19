"""SRC20 encrypted amount parsing and decryption.

Ports ``seismic-viem/src/actions/src20/crypto.ts``.

On-chain, the encrypted amount field is packed as::

    encryptedAmount = ciphertext || nonce  (last 12 bytes)

The ciphertext includes the 16-byte AES-GCM authentication tag.
Decryption uses AES-256-GCM with **no AAD** (unlike transaction
encryption which binds AAD to tx metadata).
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from hexbytes import HexBytes

from seismic_web3._types import Bytes32, EncryptionNonce
from seismic_web3.crypto.aes import AesGcmCrypto

if TYPE_CHECKING:
    pass

#: Nonce length in bytes (12 bytes = 96-bit AES-GCM nonce).
NONCE_BYTES = 12


def parse_encrypted_data(encrypted_data: bytes) -> tuple[bytes, bytes]:
    """Split an on-chain encrypted amount into ciphertext and nonce.

    Args:
        encrypted_data: Raw bytes from the ``encryptedAmount`` log field.

    Returns:
        ``(ciphertext, nonce)`` where *ciphertext* includes the 16-byte
        AES-GCM auth tag and *nonce* is exactly 12 bytes.

    Raises:
        ValueError: If the data is empty or too short.
    """
    if not encrypted_data or len(encrypted_data) <= NONCE_BYTES:
        raise ValueError(
            "Encrypted data is empty or too short — "
            "recipient may not have a registered key"
        )
    nonce = encrypted_data[-NONCE_BYTES:]
    ciphertext = encrypted_data[:-NONCE_BYTES]
    return ciphertext, nonce


def decrypt_encrypted_amount(aes_key: Bytes32, encrypted_amount: bytes) -> int:
    """Decrypt an SRC20 encrypted amount field and return the uint256 value.

    Args:
        aes_key: 32-byte AES-256 viewing key.
        encrypted_amount: Raw bytes ``ciphertext || nonce(12)``.

    Returns:
        The decrypted amount as a Python ``int``.
    """
    ciphertext, nonce_bytes = parse_encrypted_data(encrypted_amount)
    crypto = AesGcmCrypto(aes_key)
    plaintext = crypto.decrypt(
        HexBytes(ciphertext),
        EncryptionNonce(nonce_bytes),
        aad=None,
    )
    return int.from_bytes(plaintext, "big")
