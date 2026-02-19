"""Seismic network chain definitions and protocol constants.

Provides ``ChainConfig`` dataclasses for supported Seismic networks
(testnet, local sanvil) and constants used throughout the SDK.
"""

from __future__ import annotations

from dataclasses import dataclass

# ---------------------------------------------------------------------------
# Protocol constants
# ---------------------------------------------------------------------------

#: Seismic custom transaction type byte (decimal 74, hex 0x4a).
SEISMIC_TX_TYPE: int = 0x4A

#: EIP-712 typed-data message version for JSON-RPC wallet signing.
TYPED_DATA_MESSAGE_VERSION: int = 2

# ---------------------------------------------------------------------------
# Chain IDs
# ---------------------------------------------------------------------------

#: Chain ID for the Seismic public testnet.
SEISMIC_TESTNET_CHAIN_ID: int = 5124

#: Chain ID for the local Sanvil (Seismic Anvil) instance.
SANVIL_CHAIN_ID: int = 31_337

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
