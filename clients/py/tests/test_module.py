"""Tests for seismic_web3.module — SeismicNamespace and AsyncSeismicNamespace."""

from unittest.mock import MagicMock

from hexbytes import HexBytes

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
        assert hasattr(ns, "debug_send_shielded_transaction")
        assert callable(ns.get_tee_public_key)
        assert callable(ns.contract)
        assert callable(ns.send_shielded_transaction)
        assert callable(ns.signed_call)
        assert callable(ns.debug_send_shielded_transaction)

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


class TestDepositActions:
    """Test deposit action methods on SeismicNamespace."""

    def test_has_deposit_methods(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        ns = SeismicNamespace(w3, encryption, pk)

        assert callable(ns.deposit)
        assert callable(ns.get_deposit_root)
        assert callable(ns.get_deposit_count)

    def test_deposit_calls_send_transaction(self):
        w3 = MagicMock()
        w3.eth.send_transaction.return_value = HexBytes(b"\xaa" * 32)
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        ns = SeismicNamespace(w3, encryption, pk)

        tx = ns.deposit(
            b"\x01" * 32,  # node_pubkey
            b"\x02" * 48,  # consensus_pubkey
            b"\x03" * 32,  # withdrawal_credentials
            b"\x04" * 64,  # node_signature
            b"\x05" * 96,  # consensus_signature
            b"\x06" * 32,  # deposit_data_root
            value=32 * 10**18,
        )
        assert tx == HexBytes(b"\xaa" * 32)
        w3.eth.send_transaction.assert_called_once()
        call_args = w3.eth.send_transaction.call_args[0][0]
        assert call_args["value"] == 32 * 10**18
        assert "data" in call_args

    def test_get_deposit_root_calls_eth_call(self):
        w3 = MagicMock()
        # Return 32 bytes (bytes32 ABI-encoded)
        w3.eth.call.return_value = HexBytes(b"\xff" * 32)
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        ns = SeismicNamespace(w3, encryption, pk)

        root = ns.get_deposit_root()
        assert isinstance(root, bytes)
        assert len(root) == 32
        w3.eth.call.assert_called_once()

    def test_get_deposit_count_decodes_le(self):
        w3 = MagicMock()
        # ABI-encoded bytes: offset(32) + length(32) + data(8 + padding)
        offset = (32).to_bytes(32, "big")
        length = (8).to_bytes(32, "big")
        count_le = (5).to_bytes(8, "little") + b"\x00" * 24
        w3.eth.call.return_value = HexBytes(offset + length + count_le)
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        ns = SeismicNamespace(w3, encryption, pk)

        count = ns.get_deposit_count()
        assert count == 5


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
        assert hasattr(ns, "debug_send_shielded_transaction")

    def test_has_deposit_methods(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        ns = AsyncSeismicNamespace(w3, encryption, pk)

        assert callable(ns.deposit)
        assert callable(ns.get_deposit_root)
        assert callable(ns.get_deposit_count)

    def test_contract_returns_async_shielded_contract(self):
        w3 = MagicMock()
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        pk = PrivateKey(b"\x01" * 32)
        addr = "0xd3e8763675e4c425df46cc3b5c0f6cbdac396046"

        ns = AsyncSeismicNamespace(w3, encryption, pk)
        contract = ns.contract(addr, COUNTER_ABI)
        assert isinstance(contract, AsyncShieldedContract)
