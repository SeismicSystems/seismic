"""Seismic network chain definitions and protocol constants.

Provides ``ChainConfig`` dataclasses for supported Seismic networks
(testnet, local sanvil) and constants used throughout the SDK.
"""

from __future__ import annotations

import warnings
from dataclasses import dataclass
from typing import TYPE_CHECKING

from seismic_web3._constants import (
    SANVIL_CHAIN_ID,
    SEISMIC_TESTNET_CHAIN_ID,
    SEISMIC_TX_TYPE,
    TYPED_DATA_MESSAGE_VERSION,
)
from seismic_web3.client import (
    create_async_public_client,
    create_async_wallet_client,
    create_public_client,
    create_wallet_client,
)

if TYPE_CHECKING:
    from web3 import AsyncWeb3, Web3

    from seismic_web3._types import PrivateKey

# Re-export constants for backwards compatibility
__all__ = [
    "SANVIL",
    "SANVIL_CHAIN_ID",
    "SEISMIC_TESTNET",
    "SEISMIC_TESTNET_CHAIN_ID",
    "SEISMIC_TX_TYPE",
    "TYPED_DATA_MESSAGE_VERSION",
    "ChainConfig",
    "make_seismic_testnet",
]

# ---------------------------------------------------------------------------
# Chain configuration
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class ChainConfig:
    """Immutable configuration for a Seismic network.

    Attributes:
        chain_id: Numeric chain identifier.
        rpc_url: HTTP(S) JSON-RPC endpoint.
        ws_url: WebSocket endpoint (``None`` if not available).
        name: Human-readable network name.
    """

    chain_id: int
    rpc_url: str
    ws_url: str | None = None
    name: str = ""

    # -- Wallet (private key required) ----------------------------------

    def wallet_client(
        self,
        private_key: PrivateKey,
        *,
        encryption_sk: PrivateKey | None = None,
    ) -> Web3:
        """Create a sync ``Web3`` instance with wallet capabilities.

        Args:
            private_key: 32-byte signing key for transactions.
            encryption_sk: Optional 32-byte key for ECDH.

        Returns:
            A ``Web3`` instance with ``w3.seismic`` namespace attached.
        """
        return create_wallet_client(
            self.rpc_url,
            private_key=private_key,
            encryption_sk=encryption_sk,
        )

    async def async_wallet_client(
        self,
        private_key: PrivateKey,
        *,
        encryption_sk: PrivateKey | None = None,
        ws: bool = False,
    ) -> AsyncWeb3:
        """Create an async ``Web3`` instance with wallet capabilities.

        When ``ws=True`` and :attr:`ws_url` is set, the WebSocket URL
        is used automatically.

        Args:
            private_key: 32-byte signing key for transactions.
            encryption_sk: Optional 32-byte key for ECDH.
            ws: If ``True``, uses ``WebSocketProvider``.

        Returns:
            An ``AsyncWeb3`` instance with ``w3.seismic`` namespace attached.
        """
        url = self.ws_url if ws and self.ws_url else self.rpc_url
        return await create_async_wallet_client(
            url,
            private_key=private_key,
            encryption_sk=encryption_sk,
            ws=ws,
        )

    # -- Public (no private key required) --------------------------------

    def public_client(self) -> Web3:
        """Create a sync ``Web3`` instance with public (read-only) access.

        No private key required.

        Returns:
            A ``Web3`` instance with ``w3.seismic`` namespace attached
            (read-only).
        """
        return create_public_client(self.rpc_url)

    async def async_public_client(
        self,
        *,
        ws: bool = False,
    ) -> AsyncWeb3:
        """Create an async ``Web3`` instance with public (read-only) access.

        No private key required.

        Args:
            ws: If ``True``, uses ``WebSocketProvider``.

        Returns:
            An ``AsyncWeb3`` instance with ``w3.seismic`` namespace attached
            (read-only).
        """
        url = self.ws_url if ws and self.ws_url else self.rpc_url
        return await create_async_public_client(url, ws=ws)

    # -- Deprecated aliases ----------------------------------------------

    def create_client(
        self,
        private_key: PrivateKey,
        *,
        encryption_sk: PrivateKey | None = None,
    ) -> Web3:
        """Deprecated: use :meth:`wallet_client` instead."""
        warnings.warn(
            "create_client is deprecated, use wallet_client instead",
            DeprecationWarning,
            stacklevel=2,
        )
        return self.wallet_client(
            private_key,
            encryption_sk=encryption_sk,
        )

    async def create_async_client(
        self,
        private_key: PrivateKey,
        *,
        encryption_sk: PrivateKey | None = None,
        ws: bool = False,
    ) -> AsyncWeb3:
        """Deprecated: use :meth:`async_wallet_client` instead."""
        warnings.warn(
            "create_async_client is deprecated, use async_wallet_client instead",
            DeprecationWarning,
            stacklevel=2,
        )
        return await self.async_wallet_client(
            private_key,
            encryption_sk=encryption_sk,
            ws=ws,
        )


def make_seismic_testnet(n: int = 1) -> ChainConfig:
    """Create a ``ChainConfig`` for GCP testnet instance *n*.

    Args:
        n: GCP instance number (default ``1``).

    Returns:
        A ``ChainConfig`` pointing at ``gcp-{n}.seismictest.net``.
    """
    host = f"gcp-{n}.seismictest.net"
    return ChainConfig(
        chain_id=SEISMIC_TESTNET_CHAIN_ID,
        rpc_url=f"https://{host}/rpc",
        ws_url=f"wss://{host}/ws",
        name=f"Seismic Testnet (GCP-{n})",
    )


#: Default Seismic public testnet (GCP instance 1).
SEISMIC_TESTNET: ChainConfig = make_seismic_testnet(1)

#: Local Sanvil instance for development and testing.
SANVIL: ChainConfig = ChainConfig(
    chain_id=SANVIL_CHAIN_ID,
    rpc_url="http://127.0.0.1:8545",
    ws_url="ws://127.0.0.1:8545",
    name="Sanvil (local)",
)
