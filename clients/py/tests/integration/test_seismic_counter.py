"""Integration tests for SeismicCounter (shielded contract)."""

import pytest
from web3 import Web3

from seismic_web3.chains import SEISMIC_TX_TYPE
from seismic_web3.contract.abi import encode_shielded_calldata
from seismic_web3.contract.shielded import ShieldedContract
from seismic_web3.transaction_types import (
    DebugWriteResult,
    PlaintextTx,
    UnsignedSeismicTx,
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


class TestShieldedWrite:
    def test_setNumber_succeeds(self, contract: ShieldedContract, w3: Web3) -> None:
        tx_hash = contract.write.setNumber(42)
        assert len(tx_hash) == 32
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

    def test_tx_type_is_seismic(self, contract: ShieldedContract, w3: Web3) -> None:
        tx_hash = contract.write.setNumber(99)
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["type"] == SEISMIC_TX_TYPE

    def test_increment_succeeds(self, contract: ShieldedContract, w3: Web3) -> None:
        tx_hash = contract.write.increment()
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1


class TestShieldedRead:
    def test_isOdd_initial(self, contract: ShieldedContract) -> None:
        assert contract.read.isOdd() is False

    def test_isOdd_after_odd_set(self, contract: ShieldedContract, w3: Web3) -> None:
        tx = contract.write.setNumber(11)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert contract.read.isOdd() is True

    def test_isOdd_after_even_set(self, contract: ShieldedContract, w3: Web3) -> None:
        tx = contract.write.setNumber(10)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert contract.read.isOdd() is False


class TestDebugWrite:
    def test_dwrite_returns_debug_result(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """dwrite should return a DebugWriteResult with all fields populated."""
        result = contract.dwrite.setNumber(42)
        assert isinstance(result, DebugWriteResult)
        assert isinstance(result.plaintext_tx, PlaintextTx)
        assert isinstance(result.shielded_tx, UnsignedSeismicTx)
        assert len(result.tx_hash) == 32

    def test_dwrite_sends_transaction(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """dwrite should actually broadcast the transaction."""
        result = contract.dwrite.setNumber(77)
        receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert receipt["type"] == SEISMIC_TX_TYPE

    def test_dwrite_plaintext_matches_abi_encoding(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """plaintext_tx.data should match encode_shielded_calldata output."""
        result = contract.dwrite.setNumber(99)
        expected_data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [99])
        assert result.plaintext_tx.data == expected_data

    def test_dwrite_shielded_data_differs_from_plaintext(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """shielded_tx.data (encrypted) should differ from plaintext_tx.data."""
        result = contract.dwrite.setNumber(123)
        assert result.shielded_tx.data != result.plaintext_tx.data

    def test_dwrite_shielded_tx_has_seismic_elements(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """shielded_tx should have populated seismic elements."""
        result = contract.dwrite.setNumber(55)
        se = result.shielded_tx.seismic
        assert len(bytes(se.encryption_pubkey)) == 33
        assert len(bytes(se.encryption_nonce)) == 12
        assert len(bytes(se.recent_block_hash)) == 32
        assert se.expires_at_block > 0
        assert se.signed_read is False

    def test_dwrite_state_change(self, contract: ShieldedContract, w3: Web3) -> None:
        """dwrite should modify contract state (same as write)."""
        result = contract.dwrite.setNumber(11)
        w3.eth.wait_for_transaction_receipt(result.tx_hash, timeout=30)
        assert contract.read.isOdd() is True


class TestShieldedLifecycle:
    def test_full_lifecycle(self, contract: ShieldedContract, w3: Web3) -> None:
        # setNumber(11) -> isOdd == true
        w3.eth.wait_for_transaction_receipt(contract.write.setNumber(11), timeout=30)
        assert contract.read.isOdd() is True

        # increment() -> 12 -> isOdd == false
        w3.eth.wait_for_transaction_receipt(contract.write.increment(), timeout=30)
        assert contract.read.isOdd() is False
