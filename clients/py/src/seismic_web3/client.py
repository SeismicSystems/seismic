"""Seismic encryption state and web3 client factories.

Provides :class:`EncryptionState` (ECDH-derived AES key + keypair)
and factory functions to create sync/async ``Web3`` instances
pre-configured for Seismic shielded transactions.
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

from seismic_web3._types import Bytes32, CompressedPublicKey, PrivateKey
from seismic_web3.crypto.aes import AesGcmCrypto
from seismic_web3.crypto.ecdh import generate_aes_key
from seismic_web3.crypto.secp import private_key_to_compressed_public_key
from seismic_web3.transaction.aead import encode_metadata_as_aad

if TYPE_CHECKING:
    from hexbytes import HexBytes
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import EncryptionNonce
    from seismic_web3.transaction_types import TxSeismicMetadata


@dataclass
class EncryptionState:
    """Holds the AES key and encryption keypair derived from ECDH.

    Created by :func:`get_encryption` during client setup.  Pure
    computation - works in both sync and async contexts.

    Attributes:
        aes_key: 32-byte AES-256 key derived from ECDH + HKDF.
        encryption_pubkey: Client's compressed secp256k1 public key.
        encryption_private_key: Client's secp256k1 private key.
    """

    aes_key: Bytes32
    encryption_pubkey: CompressedPublicKey
    encryption_private_key: PrivateKey
    _crypto: AesGcmCrypto = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        self._crypto = AesGcmCrypto(self.aes_key)

    def encrypt(
        self,
        plaintext: HexBytes,
        nonce: EncryptionNonce,
        metadata: TxSeismicMetadata,
    ) -> HexBytes:
        """Encrypt plaintext calldata with metadata-bound AAD.

        Args:
            plaintext: Raw calldata to encrypt.
            nonce: 12-byte AES-GCM nonce.
            metadata: Transaction metadata (used to build AAD).

        Returns:
            Ciphertext with 16-byte authentication tag.
        """
        aad = encode_metadata_as_aad(metadata)
        return self._crypto.encrypt(plaintext, nonce, aad)

    def decrypt(
        self,
        ciphertext: HexBytes,
        nonce: EncryptionNonce,
        metadata: TxSeismicMetadata,
    ) -> HexBytes:
        """Decrypt ciphertext with metadata-bound AAD.

        Args:
            ciphertext: Encrypted data (includes auth tag).
            nonce: 12-byte AES-GCM nonce.
            metadata: Transaction metadata (used to build AAD).

        Returns:
            Decrypted plaintext.

        Raises:
            cryptography.exceptions.InvalidTag: If authentication fails.
        """
        aad = encode_metadata_as_aad(metadata)
        return self._crypto.decrypt(ciphertext, nonce, aad)


def get_encryption(
    network_pk: CompressedPublicKey,
    client_sk: PrivateKey | None = None,
) -> EncryptionState:
    """Derive encryption state from a TEE public key.

    Pure computation (no I/O).  If ``client_sk`` is not provided,
    a random ephemeral private key is generated.

    Args:
        network_pk: The TEE's 33-byte compressed public key.
        client_sk: Optional 32-byte client private key.  If ``None``,
            a random key is generated.

    Returns:
        Fully initialized :class:`EncryptionState`.
    """
    if client_sk is None:
        client_sk = PrivateKey(os.urandom(32))

    aes_key = generate_aes_key(client_sk, network_pk)
    client_pubkey = private_key_to_compressed_public_key(client_sk)

    return EncryptionState(
        aes_key=aes_key,
        encryption_pubkey=client_pubkey,
        encryption_private_key=client_sk,
    )


def create_shielded_web3(
    rpc_url: str,
    *,
    private_key: PrivateKey,
    encryption_sk: PrivateKey | None = None,
) -> Web3:
    """Create a sync ``Web3`` instance configured for Seismic.

    Steps:
        1. Create ``Web3`` with ``HTTPProvider(rpc_url)``.
        2. Fetch the TEE public key (sync RPC call).
        3. Derive encryption state (ECDH + HKDF).
        4. Attach :class:`~seismic_web3.module.SeismicNamespace`
           as ``w3.seismic``.
        5. Normal ``w3.eth`` transactions work unchanged.

    Args:
        rpc_url: HTTP(S) URL of the Seismic node.
        private_key: 32-byte signing key for transactions.
        encryption_sk: Optional 32-byte key for ECDH.  If ``None``,
            a random ephemeral key is generated.

    Returns:
        A ``Web3`` instance with ``w3.seismic`` namespace attached.
    """
    from web3 import Web3

    from seismic_web3.module import SeismicNamespace
    from seismic_web3.rpc import get_tee_public_key

    w3 = Web3(Web3.HTTPProvider(rpc_url))
    network_pk = get_tee_public_key(w3)
    encryption = get_encryption(network_pk, encryption_sk)

    w3.seismic = SeismicNamespace(w3, encryption, private_key)  # type: ignore[attr-defined]
    return w3


async def create_async_shielded_web3(
    provider_url: str,
    *,
    private_key: PrivateKey,
    encryption_sk: PrivateKey | None = None,
    use_websocket: bool = False,
) -> AsyncWeb3:
    """Create an async ``Web3`` instance configured for Seismic.

    Args:
        provider_url: HTTP(S) or WS(S) URL of the Seismic node.
        private_key: 32-byte signing key for transactions.
        encryption_sk: Optional 32-byte key for ECDH.  If ``None``,
            a random ephemeral key is generated.
        use_websocket: If ``True``, uses ``WebSocketProvider``
            (persistent connection, supports subscriptions).
            Otherwise uses ``AsyncHTTPProvider``.

    Returns:
        An ``AsyncWeb3`` instance with ``w3.seismic`` namespace attached.
    """
    from web3 import AsyncHTTPProvider, AsyncWeb3

    from seismic_web3.module import AsyncSeismicNamespace
    from seismic_web3.rpc import async_get_tee_public_key

    if use_websocket:
        from web3 import WebSocketProvider

        provider = WebSocketProvider(provider_url)
    else:
        provider = AsyncHTTPProvider(provider_url)

    w3 = AsyncWeb3(provider)
    network_pk = await async_get_tee_public_key(w3)
    encryption = get_encryption(network_pk, encryption_sk)

    w3.seismic = AsyncSeismicNamespace(w3, encryption, private_key)  # type: ignore[attr-defined]
    return w3
