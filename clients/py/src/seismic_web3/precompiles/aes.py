"""AES-GCM encrypt/decrypt precompiles (addresses ``0x66`` and ``0x67``).

On-chain AES-256-GCM encryption and decryption.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from hexbytes import HexBytes

from seismic_web3.precompiles._base import (
    Precompile,
    async_call_precompile,
    calc_linear_gas_cost,
    call_precompile,
)

if TYPE_CHECKING:
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import Bytes32, EncryptionNonce

AES_GCM_ENCRYPT_ADDRESS = "0x0000000000000000000000000000000000000066"
AES_GCM_DECRYPT_ADDRESS = "0x0000000000000000000000000000000000000067"

_AES_GCM_BASE_GAS = 1000
_AES_GCM_PER_BLOCK = 30  # per 16-byte block


def _nonce_to_bytes(nonce: int | EncryptionNonce) -> bytes:
    """Convert a nonce to exactly 12 bytes."""
    if isinstance(nonce, int):
        return nonce.to_bytes(12, "big")
    return bytes(nonce)


# ---------------------------------------------------------------------------
# Encryption
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class AesGcmEncryptParams:
    """Parameters for on-chain AES-GCM encryption.

    Attributes:
        aes_key: 32-byte AES-256 key.
        nonce: 12-byte nonce (int or :class:`EncryptionNonce`).
        plaintext: Data to encrypt.
    """

    aes_key: Bytes32
    nonce: int | EncryptionNonce
    plaintext: bytes


def _encrypt_gas(params: AesGcmEncryptParams) -> int:
    return calc_linear_gas_cost(
        bus=16,
        length=len(params.plaintext),
        base=_AES_GCM_BASE_GAS,
        word=_AES_GCM_PER_BLOCK,
    )


def _encrypt_encode(params: AesGcmEncryptParams) -> bytes:
    return bytes(params.aes_key) + _nonce_to_bytes(params.nonce) + params.plaintext


def _encrypt_decode(result: bytes) -> HexBytes:
    return HexBytes(result)


aes_gcm_encrypt_precompile: Precompile[AesGcmEncryptParams, HexBytes] = Precompile(
    address=AES_GCM_ENCRYPT_ADDRESS,
    gas_cost=_encrypt_gas,
    encode_params=_encrypt_encode,
    decode_result=_encrypt_decode,
)


# ---------------------------------------------------------------------------
# Decryption
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class AesGcmDecryptParams:
    """Parameters for on-chain AES-GCM decryption.

    Attributes:
        aes_key: 32-byte AES-256 key.
        nonce: 12-byte nonce (int or :class:`EncryptionNonce`).
        ciphertext: Data to decrypt (includes 16-byte auth tag).
    """

    aes_key: Bytes32
    nonce: int | EncryptionNonce
    ciphertext: bytes


def _decrypt_gas(params: AesGcmDecryptParams) -> int:
    return calc_linear_gas_cost(
        bus=16,
        length=len(params.ciphertext),
        base=_AES_GCM_BASE_GAS,
        word=_AES_GCM_PER_BLOCK,
    )


def _decrypt_encode(params: AesGcmDecryptParams) -> bytes:
    return bytes(params.aes_key) + _nonce_to_bytes(params.nonce) + params.ciphertext


def _decrypt_decode(result: bytes) -> HexBytes:
    return HexBytes(result)


aes_gcm_decrypt_precompile: Precompile[AesGcmDecryptParams, HexBytes] = Precompile(
    address=AES_GCM_DECRYPT_ADDRESS,
    gas_cost=_decrypt_gas,
    encode_params=_decrypt_encode,
    decode_result=_decrypt_decode,
)


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def aes_gcm_encrypt(
    w3: Web3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    plaintext: bytes,
) -> HexBytes:
    """On-chain AES-GCM encryption (sync).

    Args:
        w3: Sync ``Web3`` instance connected to a Seismic node.
        aes_key: 32-byte AES-256 key.
        nonce: 12-byte nonce (int or :class:`EncryptionNonce`).
        plaintext: Data to encrypt.

    Returns:
        Ciphertext bytes (includes 16-byte auth tag).
    """
    return call_precompile(
        w3,
        aes_gcm_encrypt_precompile,
        AesGcmEncryptParams(aes_key, nonce, plaintext),
    )


def aes_gcm_decrypt(
    w3: Web3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    ciphertext: bytes,
) -> HexBytes:
    """On-chain AES-GCM decryption (sync).

    Args:
        w3: Sync ``Web3`` instance connected to a Seismic node.
        aes_key: 32-byte AES-256 key.
        nonce: 12-byte nonce (int or :class:`EncryptionNonce`).
        ciphertext: Data to decrypt (includes 16-byte auth tag).

    Returns:
        Decrypted plaintext bytes.
    """
    return call_precompile(
        w3,
        aes_gcm_decrypt_precompile,
        AesGcmDecryptParams(aes_key, nonce, ciphertext),
    )


async def async_aes_gcm_encrypt(
    w3: AsyncWeb3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    plaintext: bytes,
) -> HexBytes:
    """On-chain AES-GCM encryption (async)."""
    return await async_call_precompile(
        w3,
        aes_gcm_encrypt_precompile,
        AesGcmEncryptParams(aes_key, nonce, plaintext),
    )


async def async_aes_gcm_decrypt(
    w3: AsyncWeb3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    ciphertext: bytes,
) -> HexBytes:
    """On-chain AES-GCM decryption (async)."""
    return await async_call_precompile(
        w3,
        aes_gcm_decrypt_precompile,
        AesGcmDecryptParams(aes_key, nonce, ciphertext),
    )
