"""Block explorer URL helpers for Seismic networks.

Provides utilities to generate SeismicScan explorer URLs for transactions,
addresses, tokens, and blocks, matching the TypeScript SDK implementation.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from hexbytes import HexBytes

    from seismic_web3.chains import ChainConfig

AddressExplorerTab = Literal[
    "txs",
    "token_transfers",
    "tokens",
    "internal_txns",
    "coin_balance_history",
    "logs",
    "contract",
]

TokenExplorerTab = Literal[
    "token_transfers",
    "holders",
    "contract",
]

BlockExplorerTab = Literal[
    "index",
    "txs",
]

TxExplorerTab = Literal[
    "index",
    "token_transfers",
    "internal",
    "logs",
    "state",
    "raw_trace",
]

ExplorerItem = Literal[
    "tx",
    "address",
    "token",
    "block",
]

ExplorerTab = (
    AddressExplorerTab | TokenExplorerTab | BlockExplorerTab | TxExplorerTab | str
)

__all__ = [
    "AddressExplorerTab",
    "BlockExplorerTab",
    "ExplorerItem",
    "ExplorerTab",
    "TokenExplorerTab",
    "TxExplorerTab",
    "address_explorer_url",
    "block_explorer_url",
    "get_explorer_url",
    "token_explorer_url",
    "tx_explorer_url",
]


def _format_id(val: str | int | HexBytes) -> str:
    """Format an identifier value as a string."""
    if isinstance(val, str):
        return val
    if hasattr(val, "to_0x_hex"):
        return str(val.to_0x_hex())
    if hasattr(val, "hex"):
        hex_val = str(val.hex())
        return hex_val if hex_val.startswith("0x") else f"0x{hex_val}"
    return str(val)


def get_explorer_url(
    chain: ChainConfig | None = None,
    *,
    item: ExplorerItem | None = None,
    id: str | int | HexBytes | None = None,
    tab: str | None = None,
) -> str | None:
    """Build a block explorer URL for a chain and target resource.

    Args:
        chain: Seismic network configuration. If ``None`` or if
            ``chain.explorer_url`` is not set, returns ``None``.
        item: Resource type (``'tx'``, ``'address'``, ``'token'``, or ``'block'``).
        id: Identifier for the resource (e.g. tx hash, address, or block number).
        tab: Optional sub-tab parameter (e.g. ``'logs'``, ``'token_transfers'``).

    Returns:
        The formatted explorer URL as a string, or ``None`` if no explorer
        is configured.
    """
    if chain is None or not chain.explorer_url:
        return None
    base = chain.explorer_url.rstrip("/")
    if item is None or id is None:
        return base
    formatted_id = _format_id(id)
    url = f"{base}/{item}/{formatted_id}"
    if tab:
        return f"{url}?tab={tab}"
    return url


def tx_explorer_url(
    chain: ChainConfig | None = None,
    tx_hash: str | HexBytes | None = None,
    *,
    tab: TxExplorerTab | None = None,
) -> str | None:
    """Generate a block explorer URL for a transaction.

    Args:
        chain: Seismic network configuration.
        tx_hash: Transaction hash as hex string or ``HexBytes``.
        tab: Optional tab view (``'index'``, ``'token_transfers'``, ``'internal'``,
            ``'logs'``, ``'state'``, or ``'raw_trace'``).

    Returns:
        The explorer URL for the transaction, or ``None`` if chain or explorer
        is not set.
    """
    if tx_hash is None:
        return None
    return get_explorer_url(
        chain,
        item="tx",
        id=tx_hash,
        tab=tab,
    )


def address_explorer_url(
    chain: ChainConfig | None = None,
    address: str | None = None,
    *,
    tab: AddressExplorerTab | None = None,
) -> str | None:
    """Generate a block explorer URL for an account or contract address.

    Args:
        chain: Seismic network configuration.
        address: Account or contract address.
        tab: Optional tab view (``'txs'``, ``'token_transfers'``,
            ``'tokens'``, ``'internal_txns'``, ``'coin_balance_history'``,
            ``'logs'``, or ``'contract'``).

    Returns:
        The explorer URL for the address, or ``None`` if chain or explorer
        is not set.
    """
    if address is None:
        return None
    return get_explorer_url(
        chain,
        item="address",
        id=address,
        tab=tab,
    )


def block_explorer_url(
    chain: ChainConfig | None = None,
    block_number: int | str | None = None,
    *,
    tab: BlockExplorerTab | None = None,
) -> str | None:
    """Generate a block explorer URL for a block number.

    Args:
        chain: Seismic network configuration.
        block_number: Block height as integer or string.
        tab: Optional tab view (``'index'`` or ``'txs'``).

    Returns:
        The explorer URL for the block, or ``None`` if chain or explorer is not set.
    """
    if block_number is None:
        return None
    return get_explorer_url(
        chain,
        item="block",
        id=block_number,
        tab=tab,
    )


def token_explorer_url(
    chain: ChainConfig | None = None,
    address: str | None = None,
    *,
    tab: TokenExplorerTab | None = None,
) -> str | None:
    """Generate a block explorer URL for an SRC20 or ERC20 token.

    Args:
        chain: Seismic network configuration.
        address: Token contract address.
        tab: Optional tab view (``'token_transfers'``, ``'holders'``,
            or ``'contract'``).

    Returns:
        The explorer URL for the token, or ``None`` if chain or explorer
        is not set.
    """
    if address is None:
        return None
    return get_explorer_url(
        chain,
        item="token",
        id=address,
        tab=tab,
    )
