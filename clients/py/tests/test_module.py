"""Tests for seismic_web3.module — SeismicNamespace and AsyncSeismicNamespace."""

from unittest.mock import MagicMock

from seismic_web3._types import CompressedPublicKey, PrivateKey
from seismic_web3.client import get_encryption
from seismic_web3.contract.shielded import AsyncShieldedContract, ShieldedContract
from seismic_web3.module import AsyncSeismicNamespace, SeismicNamespace

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
]


class TestSeismicNamespace:
    def test_has_expected_methods(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)

        ns = SeismicNamespace(w3, encryption, pk)

        assert hasattr(ns, "get_tee_public_key")
        assert hasattr(ns, "contract")
        assert hasattr(ns, "send_shielded_transaction")
        assert hasattr(ns, "signed_call")
        assert callable(ns.get_tee_public_key)
        assert callable(ns.contract)
        assert callable(ns.send_shielded_transaction)
        assert callable(ns.signed_call)

    def test_exposes_encryption_state(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)

        ns = SeismicNamespace(w3, encryption, pk)
        assert ns.encryption is encryption

    def test_contract_returns_shielded_contract(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        ns = SeismicNamespace(w3, encryption, pk)
        contract = ns.contract(addr, COUNTER_ABI)
        assert isinstance(contract, ShieldedContract)


class TestAsyncSeismicNamespace:
    def test_has_expected_methods(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)

        ns = AsyncSeismicNamespace(w3, encryption, pk)

        assert hasattr(ns, "get_tee_public_key")
        assert hasattr(ns, "contract")
        assert hasattr(ns, "send_shielded_transaction")
        assert hasattr(ns, "signed_call")

    def test_contract_returns_async_shielded_contract(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        ns = AsyncSeismicNamespace(w3, encryption, pk)
        contract = ns.contract(addr, COUNTER_ABI)
        assert isinstance(contract, AsyncShieldedContract)
