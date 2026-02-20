"""Public (read-only) contract interaction.

Provides :class:`PublicContract` (sync) and
:class:`AsyncPublicContract` (async) wrappers with a single namespace:

- ``.tread``  -- transparent (standard ``eth_call``)

These do not require a private key or encryption state.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from seismic_web3.contract.shielded import (
    _AsyncTransparentReadNamespace,
    _TransparentReadNamespace,
)

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from web3 import AsyncWeb3, Web3


class PublicContract:
    """Sync contract wrapper with transparent read-only access.

    Provides a single namespace for reading public contract state:

    - ``tread``  -- transparent (standard ``eth_call``)

    Example::

        contract = PublicContract(w3, addr, abi)
        result = contract.tread.getNumber()

    Args:
        w3: Sync ``Web3`` instance.
        address: Contract address.
        abi: Contract ABI (list of function entries).
    """

    def __init__(
        self,
        w3: Web3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._address = address
        self._abi = abi
        self.tread = _TransparentReadNamespace(w3, address, abi)


class AsyncPublicContract:
    """Async contract wrapper with transparent read-only access.

    Provides a single namespace for reading public contract state:

    - ``tread``  -- transparent async (standard ``eth_call``)

    Example::

        contract = AsyncPublicContract(w3, addr, abi)
        result = await contract.tread.getNumber()

    Args:
        w3: Async ``AsyncWeb3`` instance.
        address: Contract address.
        abi: Contract ABI (list of function entries).
    """

    def __init__(
        self,
        w3: AsyncWeb3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        self._w3 = w3
        self._address = address
        self._abi = abi
        self.tread = _AsyncTransparentReadNamespace(w3, address, abi)
