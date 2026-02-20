"""Seismic namespaces for ``Web3`` instances.

Provides four namespace classes attached as ``w3.seismic`` by the
factory functions in :mod:`seismic_web3.client`:

**Public (read-only, no private key):**
    :class:`SeismicPublicNamespace` (sync),
    :class:`AsyncSeismicPublicNamespace` (async)

**Wallet (full capabilities, requires private key):**
    :class:`SeismicNamespace` (sync) -- extends ``SeismicPublicNamespace``
    :class:`AsyncSeismicNamespace` (async) -- extends ``AsyncSeismicPublicNamespace``

These are plain Python objects (not web3.py ``Module`` subclasses)
for simplicity and flexibility.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from seismic_web3.abis.deposit_contract import (
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
    _check_bytes,
)
from seismic_web3.contract.abi import encode_shielded_calldata
from seismic_web3.contract.public import AsyncPublicContract, PublicContract
from seismic_web3.contract.shielded import AsyncShieldedContract, ShieldedContract
from seismic_web3.rpc import async_get_tee_public_key, get_tee_public_key
from seismic_web3.transaction.send import (
    async_debug_send_shielded_transaction,
    async_send_shielded_transaction,
    async_signed_call,
    debug_send_shielded_transaction,
    send_shielded_transaction,
    signed_call,
)

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from hexbytes import HexBytes
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import CompressedPublicKey, PrivateKey
    from seismic_web3.client import EncryptionState
    from seismic_web3.transaction_types import DebugWriteResult, SeismicSecurityParams


# ---------------------------------------------------------------------------
# Public (read-only) namespaces
# ---------------------------------------------------------------------------


class SeismicPublicNamespace:
    """Sync public Seismic namespace -- attached as ``w3.seismic``.

    Provides read-only convenience methods that do not require
    a private key or encryption state.  Created automatically by
    :func:`~seismic_web3.client.create_public_client`.

    Args:
        w3: Sync ``Web3`` instance.
    """

    def __init__(self, w3: Web3) -> None:
        self._w3 = w3

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
    ) -> PublicContract:
        """Create a :class:`PublicContract` wrapper (sync).

        Args:
            address: Contract address.
            abi: Contract ABI (list of function entries).

        Returns:
            Public contract with ``.tread`` namespace only.
        """
        return PublicContract(self._w3, address, abi)

    def get_deposit_root(
        self,
        *,
        address: str = DEPOSIT_CONTRACT_ADDRESS,
    ) -> bytes:
        """Read the current deposit Merkle root (sync).

        Args:
            address: Deposit contract address (defaults to genesis).

        Returns:
            32-byte deposit root hash.
        """
        data = encode_shielded_calldata(
            DEPOSIT_CONTRACT_ABI,
            "get_deposit_root",
            [],
        )
        raw = self._w3.eth.call({"to": address, "data": data})
        return bytes(raw[:32])

    def get_deposit_count(
        self,
        *,
        address: str = DEPOSIT_CONTRACT_ADDRESS,
    ) -> int:
        """Read the current deposit count (sync).

        The on-chain value is an 8-byte little-endian integer.

        Args:
            address: Deposit contract address (defaults to genesis).

        Returns:
            Number of deposits as a Python ``int``.
        """
        data = encode_shielded_calldata(
            DEPOSIT_CONTRACT_ABI,
            "get_deposit_count",
            [],
        )
        raw = self._w3.eth.call({"to": address, "data": data})
        count_bytes = bytes(raw[64:72])
        return int.from_bytes(count_bytes, "little")


class AsyncSeismicPublicNamespace:
    """Async public Seismic namespace -- attached as ``w3.seismic``.

    Provides read-only convenience methods that do not require
    a private key or encryption state.  Created automatically by
    :func:`~seismic_web3.client.create_async_public_client`.

    Args:
        w3: Async ``AsyncWeb3`` instance.
    """

    def __init__(self, w3: AsyncWeb3) -> None:
        self._w3 = w3

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
    ) -> AsyncPublicContract:
        """Create an :class:`AsyncPublicContract` wrapper.

        Args:
            address: Contract address.
            abi: Contract ABI (list of function entries).

        Returns:
            Async public contract with ``.tread`` namespace only.
        """
        return AsyncPublicContract(self._w3, address, abi)

    async def get_deposit_root(
        self,
        *,
        address: str = DEPOSIT_CONTRACT_ADDRESS,
    ) -> bytes:
        """Read the current deposit Merkle root (async).

        Args:
            address: Deposit contract address (defaults to genesis).

        Returns:
            32-byte deposit root hash.
        """
        data = encode_shielded_calldata(
            DEPOSIT_CONTRACT_ABI,
            "get_deposit_root",
            [],
        )
        raw = await self._w3.eth.call({"to": address, "data": data})
        return bytes(raw[:32])

    async def get_deposit_count(
        self,
        *,
        address: str = DEPOSIT_CONTRACT_ADDRESS,
    ) -> int:
        """Read the current deposit count (async).

        The on-chain value is an 8-byte little-endian integer.

        Args:
            address: Deposit contract address (defaults to genesis).

        Returns:
            Number of deposits as a Python ``int``.
        """
        data = encode_shielded_calldata(
            DEPOSIT_CONTRACT_ABI,
            "get_deposit_count",
            [],
        )
        raw = await self._w3.eth.call({"to": address, "data": data})
        count_bytes = bytes(raw[64:72])
        return int.from_bytes(count_bytes, "little")


# ---------------------------------------------------------------------------
# Wallet (full capabilities) namespaces — extend public namespaces
# ---------------------------------------------------------------------------


class SeismicNamespace(SeismicPublicNamespace):
    """Sync Seismic namespace -- attached as ``w3.seismic``.

    Extends :class:`SeismicPublicNamespace` with wallet capabilities
    that require a private key: shielded transactions, signed reads,
    debug writes, and validator deposits.

    Created automatically by
    :func:`~seismic_web3.client.create_wallet_client`.

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
        super().__init__(w3)
        self.encryption = encryption
        self._private_key = private_key

    def contract(  # type: ignore[override]
        self,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
        eip712: bool = False,
    ) -> ShieldedContract:
        """Create a :class:`ShieldedContract` wrapper (sync).

        Args:
            address: Contract address.
            abi: Contract ABI (list of function entries).
            eip712: Use EIP-712 typed data signing (default ``False``).

        Returns:
            Shielded contract with ``.write``, ``.read``,
            ``.twrite``, ``.tread``, and ``.dwrite`` namespaces.
        """
        return ShieldedContract(
            self._w3,
            self.encryption,
            self._private_key,
            address,
            abi,
            eip712=eip712,
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
        eip712: bool = False,
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
            eip712=eip712,
        )

    def signed_call(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int = 30_000_000,
        security: SeismicSecurityParams | None = None,
        eip712: bool = False,
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
            eip712=eip712,
        )

    def debug_send_shielded_transaction(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int | None = None,
        gas_price: int | None = None,
        security: SeismicSecurityParams | None = None,
        eip712: bool = False,
    ) -> DebugWriteResult:
        """Send a shielded transaction and return debug info (sync).

        Like :meth:`send_shielded_transaction` but also returns
        the plaintext and encrypted transaction views.

        Args:
            to: Recipient address.
            data: Plaintext calldata (will be encrypted).
            value: Wei to transfer (default ``0``).
            gas: Gas limit.
            gas_price: Gas price in wei.
            security: Optional security parameter overrides.

        Returns:
            :class:`~seismic_web3.transaction_types.DebugWriteResult`.
        """
        return debug_send_shielded_transaction(
            self._w3,
            encryption=self.encryption,
            private_key=self._private_key,
            to=to,
            data=data,
            value=value,
            gas=gas,
            gas_price=gas_price,
            security=security,
            eip712=eip712,
        )

    # ------------------------------------------------------------------
    # Deposit contract actions
    # ------------------------------------------------------------------

    def deposit(
        self,
        *,
        node_pubkey: bytes,
        consensus_pubkey: bytes,
        withdrawal_credentials: bytes,
        node_signature: bytes,
        consensus_signature: bytes,
        deposit_data_root: bytes,
        value: int,
        address: str = DEPOSIT_CONTRACT_ADDRESS,
    ) -> HexBytes:
        """Submit a validator deposit (sync).

        All parameters are keyword-only to prevent mix-ups between
        the many ``bytes`` arguments.

        Encodes and sends a transparent ``deposit()`` call to the
        deposit contract.  Equivalent to
        ``contract.twrite.deposit(...)`` but without instantiating
        a ``ShieldedContract``.

        Args:
            node_pubkey: 32-byte ED25519 public key.
            consensus_pubkey: 48-byte BLS12-381 public key.
            withdrawal_credentials: 32-byte withdrawal credentials.
            node_signature: 64-byte ED25519 signature.
            consensus_signature: 96-byte BLS12-381 signature.
            deposit_data_root: 32-byte deposit data root hash.
            value: Deposit amount in wei (e.g. ``32 * 10**18``).
            address: Deposit contract address (defaults to genesis).

        Returns:
            Transaction hash.

        Raises:
            ValueError: If any argument has the wrong byte length.
        """
        _check_bytes("node_pubkey", node_pubkey, 32)
        _check_bytes("consensus_pubkey", consensus_pubkey, 48)
        _check_bytes("withdrawal_credentials", withdrawal_credentials, 32)
        _check_bytes("node_signature", node_signature, 64)
        _check_bytes("consensus_signature", consensus_signature, 96)
        _check_bytes("deposit_data_root", deposit_data_root, 32)

        data = encode_shielded_calldata(
            DEPOSIT_CONTRACT_ABI,
            "deposit",
            [
                node_pubkey,
                consensus_pubkey,
                withdrawal_credentials,
                node_signature,
                consensus_signature,
                deposit_data_root,
            ],
        )
        return self._w3.eth.send_transaction(
            {"to": address, "data": data.to_0x_hex(), "value": value},
        )


class AsyncSeismicNamespace(AsyncSeismicPublicNamespace):
    """Async Seismic namespace -- attached as ``w3.seismic``.

    Extends :class:`AsyncSeismicPublicNamespace` with wallet capabilities
    that require a private key: shielded transactions, signed reads,
    debug writes, and validator deposits.

    Created automatically by
    :func:`~seismic_web3.client.create_async_wallet_client`.

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
        super().__init__(w3)
        self.encryption = encryption
        self._private_key = private_key

    def contract(  # type: ignore[override]
        self,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
        eip712: bool = False,
    ) -> AsyncShieldedContract:
        """Create an :class:`AsyncShieldedContract` wrapper.

        Args:
            address: Contract address.
            abi: Contract ABI (list of function entries).
            eip712: Use EIP-712 typed data signing (default ``False``).

        Returns:
            Async shielded contract with ``.write``, ``.read``,
            ``.twrite``, ``.tread``, and ``.dwrite`` namespaces.
        """
        return AsyncShieldedContract(
            self._w3,
            self.encryption,
            self._private_key,
            address,
            abi,
            eip712=eip712,
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
        eip712: bool = False,
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
            eip712=eip712,
        )

    async def signed_call(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int = 30_000_000,
        security: SeismicSecurityParams | None = None,
        eip712: bool = False,
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
            eip712=eip712,
        )

    async def debug_send_shielded_transaction(
        self,
        *,
        to: ChecksumAddress,
        data: HexBytes,
        value: int = 0,
        gas: int | None = None,
        gas_price: int | None = None,
        security: SeismicSecurityParams | None = None,
        eip712: bool = False,
    ) -> DebugWriteResult:
        """Send a shielded transaction and return debug info (async).

        Like :meth:`send_shielded_transaction` but also returns
        the plaintext and encrypted transaction views.

        Args:
            to: Recipient address.
            data: Plaintext calldata (will be encrypted).
            value: Wei to transfer (default ``0``).
            gas: Gas limit.
            gas_price: Gas price in wei.
            security: Optional security parameter overrides.

        Returns:
            :class:`~seismic_web3.transaction_types.DebugWriteResult`.
        """
        return await async_debug_send_shielded_transaction(
            self._w3,
            encryption=self.encryption,
            private_key=self._private_key,
            to=to,
            data=data,
            value=value,
            gas=gas,
            gas_price=gas_price,
            security=security,
            eip712=eip712,
        )

    # ------------------------------------------------------------------
    # Deposit contract actions
    # ------------------------------------------------------------------

    async def deposit(
        self,
        *,
        node_pubkey: bytes,
        consensus_pubkey: bytes,
        withdrawal_credentials: bytes,
        node_signature: bytes,
        consensus_signature: bytes,
        deposit_data_root: bytes,
        value: int,
        address: str = DEPOSIT_CONTRACT_ADDRESS,
    ) -> HexBytes:
        """Submit a validator deposit (async).

        All parameters are keyword-only to prevent mix-ups between
        the many ``bytes`` arguments.

        Encodes and sends a transparent ``deposit()`` call to the
        deposit contract.  Equivalent to
        ``await contract.twrite.deposit(...)`` but without
        instantiating an ``AsyncShieldedContract``.

        Args:
            node_pubkey: 32-byte ED25519 public key.
            consensus_pubkey: 48-byte BLS12-381 public key.
            withdrawal_credentials: 32-byte withdrawal credentials.
            node_signature: 64-byte ED25519 signature.
            consensus_signature: 96-byte BLS12-381 signature.
            deposit_data_root: 32-byte deposit data root hash.
            value: Deposit amount in wei (e.g. ``32 * 10**18``).
            address: Deposit contract address (defaults to genesis).

        Returns:
            Transaction hash.

        Raises:
            ValueError: If any argument has the wrong byte length.
        """
        _check_bytes("node_pubkey", node_pubkey, 32)
        _check_bytes("consensus_pubkey", consensus_pubkey, 48)
        _check_bytes("withdrawal_credentials", withdrawal_credentials, 32)
        _check_bytes("node_signature", node_signature, 64)
        _check_bytes("consensus_signature", consensus_signature, 96)
        _check_bytes("deposit_data_root", deposit_data_root, 32)

        data = encode_shielded_calldata(
            DEPOSIT_CONTRACT_ABI,
            "deposit",
            [
                node_pubkey,
                consensus_pubkey,
                withdrawal_credentials,
                node_signature,
                consensus_signature,
                deposit_data_root,
            ],
        )
        return await self._w3.eth.send_transaction(
            {"to": address, "data": data.to_0x_hex(), "value": value},
        )
