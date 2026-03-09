"""Integration tests for TransparentCounter (standard contract via twrite/tread)."""

import pytest
from web3 import Web3

from seismic_web3.chains import SEISMIC_TX_TYPE
from seismic_web3.contract.shielded import ShieldedContract
from tests.integration.contracts import (
    TRANSPARENT_COUNTER_ABI,
    TRANSPARENT_COUNTER_BYTECODE,
    deploy_contract,
)


@pytest.fixture
def contract(w3: Web3, plain_w3: Web3, account_address: str) -> ShieldedContract:
    """Deploy a fresh TransparentCounter and return a ShieldedContract."""
    addr = deploy_contract(plain_w3, TRANSPARENT_COUNTER_BYTECODE, account_address)
    return w3.seismic.contract(addr, TRANSPARENT_COUNTER_ABI)  # type: ignore[attr-defined]


class TestTransparentWrite:
    def test_twrite_setNumber_succeeds(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        tx_hash = contract.twrite.setNumber(42)
        assert len(tx_hash) == 32
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

    def test_twrite_tx_NOT_seismic(self, contract: ShieldedContract, w3: Web3) -> None:
        tx_hash = contract.twrite.setNumber(42)
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["type"] != SEISMIC_TX_TYPE


class TestTransparentRead:
    def test_tread_isOdd_initial(self, contract: ShieldedContract) -> None:
        assert contract.tread.isOdd() is False

    def test_tread_number_initial(self, contract: ShieldedContract) -> None:
        assert contract.tread.number() == 0

    def test_tread_after_twrite(self, contract: ShieldedContract, w3: Web3) -> None:
        tx = contract.twrite.setNumber(11)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert contract.tread.isOdd() is True


class TestTransparentLifecycle:
    def test_twrite_tread_lifecycle(self, contract: ShieldedContract, w3: Web3) -> None:
        # setNumber(7) -> isOdd == true
        w3.eth.wait_for_transaction_receipt(contract.twrite.setNumber(7), timeout=30)
        assert contract.tread.isOdd() is True

        # increment() -> 8 -> isOdd == false
        w3.eth.wait_for_transaction_receipt(contract.twrite.increment(), timeout=30)
        assert contract.tread.isOdd() is False
