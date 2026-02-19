"""Tests for seismic_web3.transaction.send — address derivation."""

from seismic_web3._types import PrivateKey
from seismic_web3.transaction.send import _address_from_key

# Anvil account #0
ANVIL_PK = PrivateKey(
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)
ANVIL_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"


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
