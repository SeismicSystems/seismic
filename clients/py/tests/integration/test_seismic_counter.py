"""Integration tests for SeismicCounter (shielded contract)."""

import pytest
from hexbytes import HexBytes
from web3 import Web3

from seismic_web3.chains import SEISMIC_TX_TYPE
from seismic_web3.contract.shielded import ShieldedContract
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


def _decode_bool(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


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
        result = contract.read.isOdd()
        assert result is not None
        assert _decode_bool(result) == 0

    def test_isOdd_after_odd_set(self, contract: ShieldedContract, w3: Web3) -> None:
        tx = contract.write.setNumber(11)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        result = contract.read.isOdd()
        assert result is not None
        assert _decode_bool(result) == 1

    def test_isOdd_after_even_set(self, contract: ShieldedContract, w3: Web3) -> None:
        tx = contract.write.setNumber(10)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        result = contract.read.isOdd()
        assert result is not None
        assert _decode_bool(result) == 0


class TestShieldedLifecycle:
    def test_full_lifecycle(self, contract: ShieldedContract, w3: Web3) -> None:
        # setNumber(11) -> isOdd == true
        w3.eth.wait_for_transaction_receipt(contract.write.setNumber(11), timeout=30)
        result = contract.read.isOdd()
        assert result is not None
        assert _decode_bool(result) == 1

        # increment() -> 12 -> isOdd == false
        w3.eth.wait_for_transaction_receipt(contract.write.increment(), timeout=30)
        result = contract.read.isOdd()
        assert result is not None
        assert _decode_bool(result) == 0
