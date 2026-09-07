"""Tests for seismic_web3.contract.public — PublicContract and AsyncPublicContract."""

from unittest.mock import AsyncMock, MagicMock

import pytest
from eth_abi import encode
from eth_hash.auto import keccak
from web3.exceptions import MismatchedABI

from seismic_web3.contract.public import AsyncPublicContract, PublicContract

COUNTER_ABI = [
    {
        "type": "function",
        "name": "getNumber",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view",
    },
]

_ADDR = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"
_ARGUMENT_ADDR = "0x000000000000000000000000000000000000dEaD"

OVERLOADED_ABI = [
    {
        "type": "function",
        "name": "lookup",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [{"name": "", "type": "bool"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "lookup",
        "inputs": [{"name": "account", "type": "address"}],
        "outputs": [{"name": "", "type": "bool"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "lookup",
        "inputs": [{"name": "secret", "type": "suint256"}],
        "outputs": [{"name": "", "type": "bool"}],
        "stateMutability": "view",
    },
]


class TestPublicContract:
    def test_has_tread(self):
        w3 = MagicMock()
        contract = PublicContract(w3, _ADDR, COUNTER_ABI)
        assert hasattr(contract, "tread")

    def test_no_write_namespaces(self):
        w3 = MagicMock()
        contract = PublicContract(w3, _ADDR, COUNTER_ABI)
        assert not hasattr(contract, "write")
        assert not hasattr(contract, "read")
        assert not hasattr(contract, "twrite")
        assert not hasattr(contract, "dwrite")

    def test_tread_callable(self):
        w3 = MagicMock()
        contract = PublicContract(w3, _ADDR, COUNTER_ABI)
        assert callable(contract.tread.getNumber)

    def test_tread_resolves_overloaded_function_from_arguments(self):
        w3 = MagicMock()
        w3.eth.call.return_value = encode(["bool"], [True])
        contract = PublicContract(w3, _ADDR, OVERLOADED_ABI)

        result = contract.tread.lookup(_ARGUMENT_ADDR)

        assert result is True
        request = w3.eth.call.call_args.args[0]
        assert bytes(request["data"][:4]) == keccak(b"lookup(address)")[:4]

    def test_tread_signature_disambiguates_same_encoding_type(self):
        w3 = MagicMock()
        w3.eth.call.return_value = encode(["bool"], [True])
        contract = PublicContract(w3, _ADDR, OVERLOADED_ABI)

        with pytest.raises(MismatchedABI, match="match multiple overloads"):
            contract.tread.lookup(42)

        result = getattr(contract.tread, "lookup(suint256)")(42)

        assert result is True
        request = w3.eth.call.call_args.args[0]
        assert bytes(request["data"][:4]) == keccak(b"lookup(suint256)")[:4]


class TestAsyncPublicContract:
    def test_has_tread(self):
        w3 = MagicMock()
        contract = AsyncPublicContract(w3, _ADDR, COUNTER_ABI)
        assert hasattr(contract, "tread")

    def test_no_write_namespaces(self):
        w3 = MagicMock()
        contract = AsyncPublicContract(w3, _ADDR, COUNTER_ABI)
        assert not hasattr(contract, "write")
        assert not hasattr(contract, "read")
        assert not hasattr(contract, "twrite")
        assert not hasattr(contract, "dwrite")

    def test_tread_callable(self):
        w3 = MagicMock()
        contract = AsyncPublicContract(w3, _ADDR, COUNTER_ABI)
        assert callable(contract.tread.getNumber)

    @pytest.mark.asyncio
    async def test_tread_resolves_overloaded_function_from_arguments(self):
        w3 = MagicMock()
        w3.eth.call = AsyncMock(return_value=encode(["bool"], [True]))
        contract = AsyncPublicContract(w3, _ADDR, OVERLOADED_ABI)

        result = await contract.tread.lookup(_ARGUMENT_ADDR)

        assert result is True
        request = w3.eth.call.call_args.args[0]
        assert bytes(request["data"][:4]) == keccak(b"lookup(address)")[:4]
