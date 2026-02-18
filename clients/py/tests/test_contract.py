"""Tests for seismic_web3.contract.shielded — ShieldedContract namespaces."""

from unittest.mock import MagicMock

from seismic_web3._types import (
    CompressedPublicKey,
    PrivateKey,
)
from seismic_web3.client import get_encryption
from seismic_web3.contract.shielded import AsyncShieldedContract, ShieldedContract

_NETWORK_PK = CompressedPublicKey(
    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
)
_CLIENT_SK = PrivateKey(
    "0xa30363336e1bb949185292a2a302de86e447d98f3a43d823c8c234d9e3e5ad77"
)

COUNTER_ABI = [
    {
        "type": "function",
        "name": "setNumber",
        "inputs": [{"name": "newNumber", "type": "suint256"}],
        "outputs": [],
        "stateMutability": "nonpayable",
    },
    {
        "type": "function",
        "name": "increment",
        "inputs": [],
        "outputs": [],
        "stateMutability": "nonpayable",
    },
]


def _make_encryption():
    return get_encryption(_NETWORK_PK, _CLIENT_SK)


class TestShieldedContract:
    def test_has_four_namespaces(self):
        """ShieldedContract should have write, read, twrite, tread."""
        w3 = MagicMock()
        encryption = _make_encryption()
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        contract = ShieldedContract(w3, encryption, pk, addr, COUNTER_ABI)

        assert hasattr(contract, "write")
        assert hasattr(contract, "read")
        assert hasattr(contract, "twrite")
        assert hasattr(contract, "tread")

    def test_write_namespace_getattr_returns_callable(self):
        """write.setNumber should return a callable."""
        w3 = MagicMock()
        encryption = _make_encryption()
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        contract = ShieldedContract(w3, encryption, pk, addr, COUNTER_ABI)
        fn = contract.write.setNumber
        assert callable(fn)

    def test_read_namespace_getattr_returns_callable(self):
        """read.setNumber should return a callable."""
        w3 = MagicMock()
        encryption = _make_encryption()
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        contract = ShieldedContract(w3, encryption, pk, addr, COUNTER_ABI)
        fn = contract.read.setNumber
        assert callable(fn)


class TestAsyncShieldedContract:
    def test_has_four_namespaces(self):
        """AsyncShieldedContract should have write, read, twrite, tread."""
        w3 = MagicMock()
        encryption = _make_encryption()
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        contract = AsyncShieldedContract(w3, encryption, pk, addr, COUNTER_ABI)

        assert hasattr(contract, "write")
        assert hasattr(contract, "read")
        assert hasattr(contract, "twrite")
        assert hasattr(contract, "tread")

    def test_write_namespace_getattr_returns_callable(self):
        """write.increment should return a callable (async version)."""
        w3 = MagicMock()
        encryption = _make_encryption()
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        contract = AsyncShieldedContract(w3, encryption, pk, addr, COUNTER_ABI)
        fn = contract.write.increment
        assert callable(fn)
