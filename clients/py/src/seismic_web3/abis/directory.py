"""Directory genesis contract ABI and address constant.

The Directory contract is deployed at a fixed genesis address on all
Seismic networks.  It stores per-user AES-256 encryption keys using
shielded storage (``suint256``).

Only the four functions needed by the Python SDK are included here,
matching the TypeScript ``DirectoryAbi`` in seismic-viem.
"""

from __future__ import annotations

from typing import Any

DIRECTORY_ADDRESS: str = "0x1000000000000000000000000000000000000004"

DIRECTORY_ABI: list[dict[str, Any]] = [
    {
        "type": "function",
        "name": "checkHasKey",
        "inputs": [{"name": "_addr", "type": "address"}],
        "outputs": [{"name": "", "type": "bool"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "keyHash",
        "inputs": [{"name": "to", "type": "address"}],
        "outputs": [{"name": "", "type": "bytes32"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "getKey",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "setKey",
        "inputs": [{"name": "_key", "type": "suint256"}],
        "outputs": [],
        "stateMutability": "nonpayable",
    },
]
