"""Seismic namespaces for ``Web3`` instances.

Provides :class:`SeismicNamespace` (sync) and
:class:`AsyncSeismicNamespace` (async) that are attached as
``w3.seismic`` by the factory functions in :mod:`seismic_web3.client`.

These are plain Python objects (not web3.py ``Module`` subclasses)
for simplicity and flexibility.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from seismic_web3.contract.shielded import AsyncShieldedContract, ShieldedContract
from seismic_web3.rpc import async_get_tee_public_key, get_tee_public_key
from seismic_web3.transaction.send import (
    async_send_shielded_transaction,
    async_signed_call,
    send_shielded_transaction,
    signed_call,
)

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from hexbytes import HexBytes
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import CompressedPublicKey, PrivateKey
    from seismic_web3.client import EncryptionState
    from seismic_web3.transaction_types import SeismicSecurityParams


class SeismicNamespace:
    """Sync Seismic namespace -- attached as ``w3.seismic``.

    Provides convenience methods for shielded operations on a sync
    ``Web3`` instance.  Created automatically by
    :func:`~seismic_web3.client.create_shielded_web3`.

    Attributes:
        encryption: The :class:`~seismic_web3.client.EncryptionState`
            holding the ECDH-derived AES key and keypair.

    Args:
        w3: Sync ``Web3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key for transactions.
    """

    def __init__(
        self,
        w3: Web3,
        encryption: EncryptionState,
        private_key: PrivateKey,
    ) -> None:
        self._w3 = w3
        self.encryption = encryption
        self._private_key = private_key

    def get_tee_public_key(self) -> CompressedPublicKey:
        """Fetch the TEE's compressed secp256k1 public key (sync).

        Returns:
            33-byte compressed public key.
        """
        return get_tee_public_key(self._w3)

    def contract(
        self,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> ShieldedContract:
        """Create a :class:`ShieldedContract` wrapper (sync).

        Args:
            address: Contract address.
            abi: Contract ABI (list of function entries).

        Returns:
            Shielded contract with ``.write``, ``.read``,
            ``.twrite``, and ``.tread`` namespaces.
        """
        return ShieldedContract(
            self._w3,
            self.encryption,
            self._private_key,
            address,
            abi,
        )

    def send_shielded_transaction(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int | None = None,
        gas_price: int | None = None,
        security: SeismicSecurityParams | None = None,
    ) -> HexBytes:
        """Send a shielded transaction (sync).

        Delegates to
        :func:`~seismic_web3.transaction.send.send_shielded_transaction`
        with this namespace's encryption state and private key.

        Args:
            to: Recipient address.
            data: Plaintext calldata (will be encrypted).
            value: Wei to transfer (default ``0``).
            gas: Gas limit.
            gas_price: Gas price in wei.
            security: Optional security parameter overrides.

        Returns:
            Transaction hash.
        """
        return send_shielded_transaction(
            self._w3,
            encryption=self.encryption,
            private_key=self._private_key,
            to=to,
            data=data,
            value=value,
            gas=gas,
            gas_price=gas_price,
            security=security,
        )

    def signed_call(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int = 30_000_000,
        security: SeismicSecurityParams | None = None,
    ) -> HexBytes | None:
        """Execute a signed read (sync).

        Delegates to
        :func:`~seismic_web3.transaction.send.signed_call`
        with this namespace's encryption state and private key.

        Args:
            to: Contract address to call.
            data: Plaintext calldata (will be encrypted).
            value: Wei to include (default ``0``).
            gas: Gas limit (default ``30_000_000``).
            security: Optional security parameter overrides.

        Returns:
            Decrypted response bytes, or ``None`` if empty.
        """
        return signed_call(
            self._w3,
            encryption=self.encryption,
            private_key=self._private_key,
            to=to,
            data=data,
            value=value,
            gas=gas,
            security=security,
        )


class AsyncSeismicNamespace:
    """Async Seismic namespace -- attached as ``w3.seismic``.

    Provides convenience methods for shielded operations on an async
    ``AsyncWeb3`` instance.  Created automatically by
    :func:`~seismic_web3.client.create_async_shielded_web3`.

    Attributes:
        encryption: The :class:`~seismic_web3.client.EncryptionState`
            holding the ECDH-derived AES key and keypair.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key for transactions.
    """

    def __init__(
        self,
        w3: AsyncWeb3,
        encryption: EncryptionState,
        private_key: PrivateKey,
    ) -> None:
        self._w3 = w3
        self.encryption = encryption
        self._private_key = private_key

    async def get_tee_public_key(self) -> CompressedPublicKey:
        """Fetch the TEE's compressed secp256k1 public key (async).

        Returns:
            33-byte compressed public key.
        """
        return await async_get_tee_public_key(self._w3)

    def contract(
        self,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> AsyncShieldedContract:
        """Create an :class:`AsyncShieldedContract` wrapper.

        Args:
            address: Contract address.
            abi: Contract ABI (list of function entries).

        Returns:
            Async shielded contract with ``.write``, ``.read``,
            ``.twrite``, and ``.tread`` namespaces.
        """
        return AsyncShieldedContract(
            self._w3,
            self.encryption,
            self._private_key,
            address,
            abi,
        )

    async def send_shielded_transaction(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int | None = None,
        gas_price: int | None = None,
        security: SeismicSecurityParams | None = None,
    ) -> HexBytes:
        """Send a shielded transaction (async).

        Delegates to
        :func:`~seismic_web3.transaction.send.async_send_shielded_transaction`
        with this namespace's encryption state and private key.

        Args:
            to: Recipient address.
            data: Plaintext calldata (will be encrypted).
            value: Wei to transfer (default ``0``).
            gas: Gas limit.
            gas_price: Gas price in wei.
            security: Optional security parameter overrides.

        Returns:
            Transaction hash.
        """
        return await async_send_shielded_transaction(
            self._w3,
            encryption=self.encryption,
            private_key=self._private_key,
            to=to,
            data=data,
            value=value,
            gas=gas,
            gas_price=gas_price,
            security=security,
        )

    async def signed_call(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int = 30_000_000,
        security: SeismicSecurityParams | None = None,
    ) -> HexBytes | None:
        """Execute a signed read (async).

        Delegates to
        :func:`~seismic_web3.transaction.send.async_signed_call`
        with this namespace's encryption state and private key.

        Args:
            to: Contract address to call.
            data: Plaintext calldata (will be encrypted).
            value: Wei to include (default ``0``).
            gas: Gas limit (default ``30_000_000``).
            security: Optional security parameter overrides.

        Returns:
            Decrypted response bytes, or ``None`` if empty.
        """
        return await async_signed_call(
            self._w3,
            encryption=self.encryption,
            private_key=self._private_key,
            to=to,
            data=data,
            value=value,
            gas=gas,
            security=security,
        )
