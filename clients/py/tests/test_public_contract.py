"""Tests for seismic_web3.contract.public — PublicContract and AsyncPublicContract."""

from unittest.mock import MagicMock

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
