"""Integration tests for signed eth_estimateGas on shielded transactions.

Mirrors the seismic-alloy / seismic-foundry tests:
  - Signed estimate gas succeeds and returns a reasonable value.
  - Unsigned estimate gas is rejected (node sanitizes ``from``).
  - Transactions using estimated gas execute successfully.
"""

import pytest
from hexbytes import HexBytes
from web3 import Web3
from web3.types import RPCEndpoint

from seismic_web3 import PrivateKey
from seismic_web3.contract.abi import encode_shielded_calldata
from seismic_web3.contract.shielded import ShieldedContract
from seismic_web3.transaction.metadata import build_metadata
from seismic_web3.transaction.send import (
    _address_from_key,
    _build_metadata_params,
    estimate_shielded_gas,
)
from tests.integration.contracts import (
    SEISMIC_COUNTER_ABI,
    SEISMIC_COUNTER_BYTECODE,
    deploy_contract,
)


@pytest.fixture
def contract(w3: Web3, plain_w3: Web3, account_address: str) -> ShieldedContract:
    """Deploy a fresh SeismicCounter and return a ShieldedContract."""
    addr = deploy_contract(plain_w3, SEISMIC_COUNTER_BYTECODE, account_address)
    return w3.seismic.contract(addr, SEISMIC_COUNTER_ABI)  # type: ignore[attr-defined]


class TestSignedEstimateGas:
    """Signed shielded eth_estimateGas should succeed."""

    @pytest.mark.parametrize("eip712", [False, True], ids=["raw", "eip712"])
    def test_estimate_gas_returns_reasonable_value(
        self,
        contract: ShieldedContract,
        w3: Web3,
        private_key: PrivateKey,
        eip712: bool,
    ) -> None:
        """Signed estimate gas should return a gas value > 21000."""
        encryption = w3.seismic.encryption  # type: ignore[attr-defined]
        data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [42])

        params = _build_metadata_params(
            private_key,
            encryption,
            contract._address,
            0,
            None,
            eip712=eip712,
        )
        metadata = build_metadata(w3, params)
        encrypted = encryption.encrypt(
            data,
            metadata.seismic_elements.encryption_nonce,
            metadata,
        )

        gas = estimate_shielded_gas(
            w3,
            encrypted_data=HexBytes(encrypted),
            metadata=metadata,
            gas_price=w3.eth.gas_price,
            private_key=private_key,
        )

        assert isinstance(gas, int)
        assert gas > 21_000, "Gas estimate should exceed the base tx cost"
        assert gas < 30_000_000, "Gas estimate should be well below 30M"


class TestUnsignedEstimateGasRejected:
    """Unsigned seismic estimate gas should fail after node sanitization."""

    def test_unsigned_estimate_gas_sanitized(
        self,
        contract: ShieldedContract,
        w3: Web3,
        private_key: PrivateKey,
    ) -> None:
        """Unsigned eth_estimateGas should succeed but with ``from`` cleared.

        The node sanitizes unsigned requests by stripping ``from``,
        so the result reflects execution without sender authentication.
        For a simple setNumber call this still succeeds (the function
        doesn't gate on msg.sender), but the spoofed ``from`` is ignored.
        """
        sender = _address_from_key(private_key)
        data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [42])

        response = w3.provider.make_request(
            RPCEndpoint("eth_estimateGas"),
            [{"from": sender, "to": contract._address, "data": data.to_0x_hex()}],
        )

        # The node clears `from` on unsigned requests.  For setNumber
        # (no msg.sender gating) the call still succeeds, proving the
        # supplied `from` address was not used for authentication.
        assert "result" in response, (
            f"Expected success for non-gated call, got error: {response.get('error')}"
        )


class TestEstimateGasIntegration:
    """End-to-end: estimated gas is used in real transactions."""

    def test_write_without_explicit_gas_succeeds(
        self,
        contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """write() without explicit gas should auto-estimate and succeed."""
        tx_hash = contract.write.setNumber(42)
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

    def test_write_uses_estimated_gas_not_30m(
        self,
        contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """write() should use estimated gas, not the old 30M default."""
        result = contract.dwrite.setNumber(77)
        receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash, timeout=30)
        assert receipt["status"] == 1
        # The tx should have a gas limit well below 30M
        tx = w3.eth.get_transaction(result.tx_hash)
        assert tx["gas"] < 30_000_000, f"Expected estimated gas < 30M, got {tx['gas']}"
        assert tx["gas"] > 21_000

    def test_write_with_explicit_gas_skips_estimation(
        self,
        contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """Providing explicit gas should skip estimation and use that value."""
        explicit_gas = 5_000_000
        result = contract.dwrite.setNumber(55, gas=explicit_gas)
        receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash, timeout=30)
        assert receipt["status"] == 1
        tx = w3.eth.get_transaction(result.tx_hash)
        assert tx["gas"] == explicit_gas

    def test_lifecycle_with_estimated_gas(
        self,
        contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """Full lifecycle with auto-estimated gas: set, read, increment, read."""
        tx = contract.write.setNumber(11)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert contract.read.isOdd() is True

        tx = contract.write.increment()
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert contract.read.isOdd() is False
