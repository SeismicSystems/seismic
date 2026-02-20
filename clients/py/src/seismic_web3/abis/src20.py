"""ISRC20 interface ABI -- Seismic's privacy-preserving ERC20 standard.

The SRC20 token standard uses shielded types (``suint256``) for amounts
to preserve balance and transfer privacy. Unlike ERC20, ``balanceOf()``
takes no arguments and returns the caller's own balance.

This ABI matches the ``ISRC20`` interface defined in
``contracts/src/seismic-std-lib/SRC20.sol``.
"""

from __future__ import annotations

from typing import Any

SRC20_ABI: list[dict[str, Any]] = [
    {
        "type": "function",
        "name": "name",
        "inputs": [],
        "outputs": [{"name": "", "type": "string", "internalType": "string"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "symbol",
        "inputs": [],
        "outputs": [{"name": "", "type": "string", "internalType": "string"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "decimals",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint8", "internalType": "uint8"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "balanceOf",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256", "internalType": "uint256"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "approve",
        "inputs": [
            {"name": "spender", "type": "address", "internalType": "address"},
            {"name": "amount", "type": "suint256", "internalType": "suint256"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "nonpayable",
    },
    {
        "type": "function",
        "name": "transfer",
        "inputs": [
            {"name": "to", "type": "address", "internalType": "address"},
            {"name": "amount", "type": "suint256", "internalType": "suint256"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "nonpayable",
    },
    {
        "type": "function",
        "name": "transferFrom",
        "inputs": [
            {"name": "from", "type": "address", "internalType": "address"},
            {"name": "to", "type": "address", "internalType": "address"},
            {"name": "amount", "type": "suint256", "internalType": "suint256"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "nonpayable",
    },
    # -- Events (encrypted amounts via Directory/Intelligence) ---------------
    {
        "type": "event",
        "name": "Transfer",
        "inputs": [
            {
                "name": "from",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "to",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "encryptKeyHash",
                "type": "bytes32",
                "indexed": True,
                "internalType": "bytes32",
            },
            {
                "name": "encryptedAmount",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
        ],
        "anonymous": False,
    },
    {
        "type": "event",
        "name": "Approval",
        "inputs": [
            {
                "name": "owner",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "spender",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "encryptKeyHash",
                "type": "bytes32",
                "indexed": True,
                "internalType": "bytes32",
            },
            {
                "name": "encryptedAmount",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
        ],
        "anonymous": False,
    },
]
