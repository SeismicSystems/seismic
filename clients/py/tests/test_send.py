"""Tests for seismic_web3.transaction.send — address derivation, estimation."""

from unittest.mock import MagicMock

from seismic_web3._types import CompressedPublicKey, PrivateKey
from seismic_web3.client import get_encryption
from seismic_web3.transaction.send import (
    _address_from_key,
    estimate_transparent_gas,
)

# Anvil account #0
ANVIL_PK = PrivateKey(
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)
ANVIL_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

_NETWORK_PK = CompressedPublicKey(
    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
)
_CLIENT_SK = PrivateKey(
    "0xa30363336e1bb949185292a2a302de86e447d98f3a43d823c8c234d9e3e5ad77"
)


class TestAddressFromKey:
    def test_anvil_account_0(self):
        """Derive address from Anvil's well-known account #0 private key."""
        address = _address_from_key(ANVIL_PK)
        assert address.lower() == ANVIL_ADDRESS.lower()

    def test_returns_checksummed(self):
        """Address should be checksummed (mixed case)."""
        address = _address_from_key(ANVIL_PK)
        # Checksummed addresses have uppercase letters
        assert any(c.isupper() for c in address[2:])


class TestEstimateTransparentGas:
    """Transparent gas estimation must go through a Seismic (0x4a) tx.

    The node's raw-bytes ``eth_estimateGas`` path rejects plain signed
    transactions (a signed read must carry ``seismic_elements``), and
    unsigned estimation strips ``from`` and ``value``, which breaks
    payable and sender-dependent calls.  The only request form that
    preserves authenticated caller context is a provisional shielded
    transaction.
    """

    def test_submits_seismic_tx_to_estimate_gas(self):
        w3 = MagicMock()
        w3.eth.chain_id = 31337
        w3.eth.get_transaction_count.return_value = 7
        w3.eth.get_block.return_value = {
            "hash": b"\x11" * 32,
            "number": 100,
            "gasLimit": 30_000_000,
        }
        w3.eth.gas_price = 10**9
        w3.provider.make_request.return_value = {"result": "0x5208"}

        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        gas = estimate_transparent_gas(
            w3,
            to="0x5FbDB2315678afecb367f032d93F642f64180aa3",
            data="0xd09de08a",
            value=32 * 10**18,
            private_key=ANVIL_PK,
            encryption=encryption,
        )

        assert gas == 0x5208
        method, params = w3.provider.make_request.call_args[0]
        assert method == "eth_estimateGas"
        assert params[0].startswith("0x4a"), (
            "transparent estimation must submit a provisional Seismic "
            "(0x4a) transaction: the node rejects plain signed txs on the "
            "raw-bytes path and strips from/value from unsigned requests"
        )
