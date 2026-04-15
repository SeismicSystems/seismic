"""Integration tests for smart routing (auto-detect shielded params)."""

import pytest
from web3 import Web3

from seismic_web3.chains import SEISMIC_TX_TYPE
from seismic_web3.contract.shielded import ShieldedContract
from tests.integration.contracts import (
    SEISMIC_COUNTER_ABI,
    SEISMIC_COUNTER_BYTECODE,
    TRANSPARENT_COUNTER_ABI,
    TRANSPARENT_COUNTER_BYTECODE,
    deploy_contract,
)


@pytest.fixture
def seismic_contract(
    w3: Web3,
    plain_w3: Web3,
    account_address: str,
) -> ShieldedContract:
    """Deploy a fresh SeismicCounter and return a ShieldedContract."""
    addr = deploy_contract(plain_w3, SEISMIC_COUNTER_BYTECODE, account_address)
    return w3.seismic.contract(addr, SEISMIC_COUNTER_ABI)  # type: ignore[attr-defined]


@pytest.fixture
def transparent_contract(
    w3: Web3,
    plain_w3: Web3,
    account_address: str,
) -> ShieldedContract:
    """Deploy a fresh TransparentCounter and return a ShieldedContract."""
    addr = deploy_contract(plain_w3, TRANSPARENT_COUNTER_BYTECODE, account_address)
    return w3.seismic.contract(addr, TRANSPARENT_COUNTER_ABI)  # type: ignore[attr-defined]


class TestSmartWriteRouting:
    def test_shielded_function_uses_seismic_tx(
        self,
        seismic_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """contract.write.setNumber(42) on seismic counter -> seismic tx type."""
        tx_hash = seismic_contract.write.setNumber(42)
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert receipt["type"] == SEISMIC_TX_TYPE

    def test_transparent_function_not_seismic_tx(
        self,
        seismic_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """contract.write.increment() on seismic counter -> NOT seismic tx type."""
        tx_hash = seismic_contract.write.increment()
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert receipt["type"] != SEISMIC_TX_TYPE

    def test_transparent_counter_not_seismic_tx(
        self,
        transparent_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """Transparent counter write uses non-seismic tx."""
        tx_hash = transparent_contract.write.setNumber(42)
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert receipt["type"] != SEISMIC_TX_TYPE


class TestSmartReadRouting:
    def test_read_view_no_shielded_inputs(
        self,
        seismic_contract: ShieldedContract,
    ) -> None:
        """Smart read with no shielded inputs works."""
        assert seismic_contract.read.isOdd() is False

    def test_read_after_write(
        self,
        seismic_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """After smart write, smart read returns correct value."""
        tx = seismic_contract.write.setNumber(11)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert seismic_contract.read.isOdd() is True


class TestForceShielded:
    def test_swrite_increment_is_seismic_tx(
        self,
        seismic_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """Force shielded write is seismic tx type."""
        tx_hash = seismic_contract.swrite.increment()
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert receipt["type"] == SEISMIC_TX_TYPE

    def test_sread_isOdd_works(
        self,
        seismic_contract: ShieldedContract,
    ) -> None:
        """contract.sread.isOdd() works via signed read."""
        assert seismic_contract.sread.isOdd() is False


class TestSmartRoutingLifecycle:
    def test_full_lifecycle(
        self,
        seismic_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        """End-to-end: smart write/read, force shielded, force transparent."""
        # smart write setNumber(11) -> smart read isOdd (true)
        tx = seismic_contract.write.setNumber(11)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert seismic_contract.read.isOdd() is True

        # smart write increment -> 12 -> smart read isOdd (false)
        tx = seismic_contract.write.increment()
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert seismic_contract.read.isOdd() is False

        # swrite increment -> 13 -> sread isOdd (true)
        tx = seismic_contract.swrite.increment()
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert seismic_contract.sread.isOdd() is True

        # twrite increment -> 14 -> tread isOdd (false)
        tx = seismic_contract.twrite.increment()
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert seismic_contract.tread.isOdd() is False
