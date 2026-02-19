"""Shielded contract interaction.

Provides :class:`ShieldedContract` (sync) and
:class:`AsyncShieldedContract` (async) wrappers with four namespaces:

- ``.write``  -- encrypted calldata via ``TxSeismic``
- ``.read``   -- encrypted calldata via signed ``eth_call``
- ``.twrite`` -- transparent (standard ``eth_sendTransaction``)
- ``.tread``  -- transparent (standard ``eth_call``)
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from hexbytes import HexBytes

from seismic_web3.contract.abi import encode_shielded_calldata
from seismic_web3.transaction.send import (
    async_debug_send_shielded_transaction,
    async_send_shielded_transaction,
    async_signed_call,
    debug_send_shielded_transaction,
    send_shielded_transaction,
    signed_call,
)

if TYPE_CHECKING:
    from collections.abc import Callable

    from eth_typing import ChecksumAddress
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import PrivateKey
    from seismic_web3.client import EncryptionState
    from seismic_web3.transaction_types import DebugWriteResult, SeismicSecurityParams


# ---------------------------------------------------------------------------
# Sync namespaces
# ---------------------------------------------------------------------------


class _ShieldedWriteNamespace:
    """``contract.write.functionName(*args)`` -- encrypted write (sync)."""

    def __init__(
        self,
        w3: Web3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., HexBytes]:
        """Return a callable that sends a shielded write for ``fn_name``."""

        def call(
            *args: Any,
            value: int = 0,
            gas: int | None = None,
            gas_price: int | None = None,
            security: SeismicSecurityParams | None = None,
        ) -> HexBytes:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            return send_shielded_transaction(
                self._w3,
                encryption=self._encryption,
                private_key=self._private_key,
                to=self._address,
                data=data,
                value=value,
                gas=gas,
                gas_price=gas_price,
                security=security,
            )

        return call


class _ShieldedDebugWriteNamespace:
    """``contract.dwrite.functionName(*args)`` -- debug encrypted write (sync)."""

    def __init__(
        self,
        w3: Web3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., DebugWriteResult]:
        """Return a callable that sends a shielded write and returns debug info."""

        def call(
            *args: Any,
            value: int = 0,
            gas: int | None = None,
            gas_price: int | None = None,
            security: SeismicSecurityParams | None = None,
        ) -> DebugWriteResult:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            return debug_send_shielded_transaction(
                self._w3,
                encryption=self._encryption,
                private_key=self._private_key,
                to=self._address,
                data=data,
                value=value,
                gas=gas,
                gas_price=gas_price,
                security=security,
            )

        return call


class _ShieldedReadNamespace:
    """``contract.read.functionName(*args)`` -- encrypted read (sync)."""

    def __init__(
        self,
        w3: Web3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., HexBytes | None]:
        """Return a callable that executes a signed read for ``fn_name``."""

        def call(
            *args: Any,
            value: int = 0,
            gas: int = 30_000_000,
            security: SeismicSecurityParams | None = None,
        ) -> HexBytes | None:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            return signed_call(
                self._w3,
                encryption=self._encryption,
                private_key=self._private_key,
                to=self._address,
                data=data,
                value=value,
                gas=gas,
                security=security,
            )

        return call


class _TransparentWriteNamespace:
    """``contract.twrite.functionName(*args)`` -- standard transact (sync)."""

    def __init__(
        self,
        w3: Web3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., HexBytes]:
        """Return a callable that sends a standard transaction for ``fn_name``."""

        def call(*args: Any, value: int = 0, **tx_params: Any) -> HexBytes:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            tx: dict[str, Any] = {
                "to": self._address,
                "data": data.to_0x_hex(),
                "value": value,
                **tx_params,
            }
            return self._w3.eth.send_transaction(tx)

        return call


class _TransparentReadNamespace:
    """``contract.tread.functionName(*args)`` -- standard call (sync)."""

    def __init__(
        self,
        w3: Web3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., HexBytes]:
        """Return a callable that performs a standard eth_call for ``fn_name``."""

        def call(*args: Any) -> HexBytes:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            result = self._w3.eth.call({"to": self._address, "data": data})
            return HexBytes(result)

        return call


# ---------------------------------------------------------------------------
# Async namespaces
# ---------------------------------------------------------------------------


class _AsyncShieldedWriteNamespace:
    """``contract.write.functionName(*args)`` -- encrypted write (async)."""

    def __init__(
        self,
        w3: AsyncWeb3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., Any]:
        """Return an async callable that sends a shielded write."""

        async def call(
            *args: Any,
            value: int = 0,
            gas: int | None = None,
            gas_price: int | None = None,
            security: SeismicSecurityParams | None = None,
        ) -> HexBytes:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            return await async_send_shielded_transaction(
                self._w3,
                encryption=self._encryption,
                private_key=self._private_key,
                to=self._address,
                data=data,
                value=value,
                gas=gas,
                gas_price=gas_price,
                security=security,
            )

        return call


class _AsyncShieldedDebugWriteNamespace:
    """``contract.dwrite.functionName(*args)`` -- debug encrypted write (async)."""

    def __init__(
        self,
        w3: AsyncWeb3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., Any]:
        """Return an async callable for a debug shielded write."""

        async def call(
            *args: Any,
            value: int = 0,
            gas: int | None = None,
            gas_price: int | None = None,
            security: SeismicSecurityParams | None = None,
        ) -> DebugWriteResult:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            return await async_debug_send_shielded_transaction(
                self._w3,
                encryption=self._encryption,
                private_key=self._private_key,
                to=self._address,
                data=data,
                value=value,
                gas=gas,
                gas_price=gas_price,
                security=security,
            )

        return call


class _AsyncShieldedReadNamespace:
    """``contract.read.functionName(*args)`` -- encrypted read (async)."""

    def __init__(
        self,
        w3: AsyncWeb3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., Any]:
        """Return an async callable that executes a signed read."""

        async def call(
            *args: Any,
            value: int = 0,
            gas: int = 30_000_000,
            security: SeismicSecurityParams | None = None,
        ) -> HexBytes | None:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            return await async_signed_call(
                self._w3,
                encryption=self._encryption,
                private_key=self._private_key,
                to=self._address,
                data=data,
                value=value,
                gas=gas,
                security=security,
            )

        return call


class _AsyncTransparentWriteNamespace:
    """``contract.twrite.functionName(*args)`` -- standard transact (async)."""

    def __init__(
        self,
        w3: AsyncWeb3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., Any]:
        """Return an async callable that sends a standard transaction."""

        async def call(*args: Any, value: int = 0, **tx_params: Any) -> HexBytes:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            tx: dict[str, Any] = {
                "to": self._address,
                "data": data.to_0x_hex(),
                "value": value,
                **tx_params,
            }
            return await self._w3.eth.send_transaction(tx)

        return call


class _AsyncTransparentReadNamespace:
    """``contract.tread.functionName(*args)`` -- standard call (async)."""

    def __init__(
        self,
        w3: AsyncWeb3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._address = address
        self._abi = abi

    def __getattr__(self, fn_name: str) -> Callable[..., Any]:
        """Return an async callable that performs a standard eth_call."""

        async def call(*args: Any) -> HexBytes:
            data = encode_shielded_calldata(self._abi, fn_name, list(args))
            result = await self._w3.eth.call({"to": self._address, "data": data})
            return HexBytes(result)

        return call


# ---------------------------------------------------------------------------
# Contract wrappers
# ---------------------------------------------------------------------------


class ShieldedContract:
    """Sync contract wrapper with shielded and transparent namespaces.

    Provides five namespaces for interacting with a Seismic contract:

    - ``write``  -- encrypted calldata via ``TxSeismic``
    - ``read``   -- encrypted calldata via signed ``eth_call``
    - ``twrite`` -- transparent (standard ``eth_sendTransaction``)
    - ``tread``  -- transparent (standard ``eth_call``)
    - ``dwrite`` -- debug write: like ``write`` but returns debug info

    Example::

        contract = ShieldedContract(w3, encryption, pk, addr, abi)
        tx_hash = contract.write.setNumber(42)
        result = contract.read.getNumber()

    Args:
        w3: Sync ``Web3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        address: Contract address.
        abi: Contract ABI (list of function entries).
    """

    def __init__(
        self,
        w3: Web3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

        self.write = _ShieldedWriteNamespace(w3, encryption, private_key, address, abi)
        self.read = _ShieldedReadNamespace(w3, encryption, private_key, address, abi)
        self.twrite = _TransparentWriteNamespace(w3, address, abi)
        self.tread = _TransparentReadNamespace(w3, address, abi)
        self.dwrite = _ShieldedDebugWriteNamespace(
            w3, encryption, private_key, address, abi
        )


class AsyncShieldedContract:
    """Async contract wrapper with shielded and transparent namespaces.

    Provides five namespaces for interacting with a Seismic contract:

    - ``write``  -- encrypted calldata via ``TxSeismic`` (returns coroutine)
    - ``read``   -- encrypted calldata via signed ``eth_call`` (returns coroutine)
    - ``twrite`` -- transparent async ``eth_sendTransaction``
    - ``tread``  -- transparent async ``eth_call``
    - ``dwrite`` -- debug write: like ``write`` but returns debug info

    Example::

        contract = AsyncShieldedContract(w3, encryption, pk, addr, abi)
        tx_hash = await contract.write.setNumber(42)
        result = await contract.read.getNumber()

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        address: Contract address.
        abi: Contract ABI (list of function entries).
    """

    def __init__(
        self,
        w3: AsyncWeb3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._encryption = encryption
        self._private_key = private_key
        self._address = address
        self._abi = abi

        self.write = _AsyncShieldedWriteNamespace(
            w3, encryption, private_key, address, abi
        )
        self.read = _AsyncShieldedReadNamespace(
            w3, encryption, private_key, address, abi
        )
        self.twrite = _AsyncTransparentWriteNamespace(w3, address, abi)
        self.tread = _AsyncTransparentReadNamespace(w3, address, abi)
        self.dwrite = _AsyncShieldedDebugWriteNamespace(
            w3, encryption, private_key, address, abi
        )
