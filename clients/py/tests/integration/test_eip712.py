"""Integration tests for EIP-712 typed data signing through the full pipeline.

Mirrors test_seismic_counter.py and test_namespace.py but uses ``eip712=True``
so that every transaction is signed with EIP-712 (message_version=2) instead
of the default raw signing (message_version=0).
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from seismic_web3.chains import SEISMIC_TX_TYPE
from seismic_web3.contract.abi import encode_shielded_calldata
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

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from hexbytes import HexBytes
    from web3 import Web3

    from seismic_web3.contract.shielded import ShieldedContract


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def contract(w3: Web3, plain_w3: Web3, account_address: str) -> ShieldedContract:
    """Deploy a fresh SeismicCounter and return a ShieldedContract with eip712=True."""
    addr = deploy_contract(plain_w3, SEISMIC_COUNTER_BYTECODE, account_address)
    return w3.seismic.contract(addr, SEISMIC_COUNTER_ABI, eip712=True)  # type: ignore[attr-defined]


def _deploy(plain_w3: Web3, account_address: str) -> ChecksumAddress:
    return deploy_contract(plain_w3, SEISMIC_COUNTER_BYTECODE, account_address)


def _decode_bool(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


# ===================================================================
# ShieldedContract with eip712=True
# ===================================================================


class TestEIP712ShieldedWrite:
    """Shielded writes via contract.write with EIP-712 signing."""

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


class TestEIP712ShieldedRead:
    """Shielded reads via contract.read with EIP-712 signing."""

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


class TestEIP712DebugWrite:
    """Debug writes via contract.dwrite with EIP-712 signing."""

    def test_dwrite_returns_debug_result(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        result = contract.dwrite.setNumber(42)
        assert isinstance(result, DebugWriteResult)
        assert isinstance(result.plaintext_tx, PlaintextTx)
        assert isinstance(result.shielded_tx, UnsignedSeismicTx)
        assert len(result.tx_hash) == 32

    def test_dwrite_sends_transaction(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        result = contract.dwrite.setNumber(77)
        receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert receipt["type"] == SEISMIC_TX_TYPE

    def test_dwrite_message_version_is_2(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """The shielded tx built with eip712=True must have message_version=2."""
        result = contract.dwrite.setNumber(55)
        assert result.shielded_tx.seismic.message_version == 2


class TestEIP712Lifecycle:
    """Full write-then-read lifecycle with EIP-712 signing."""

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


# ===================================================================
# w3.seismic namespace with eip712=True
# ===================================================================


class TestEIP712NamespaceSend:
    """Low-level w3.seismic.send_shielded_transaction with eip712=True."""

    def test_returns_hash(self, w3: Web3, plain_w3: Web3, account_address: str) -> None:
        addr = _deploy(plain_w3, account_address)
        data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [42])
        tx_hash = w3.seismic.send_shielded_transaction(to=addr, data=data, eip712=True)  # type: ignore[attr-defined]
        assert len(tx_hash) == 32

    def test_tx_type(self, w3: Web3, plain_w3: Web3, account_address: str) -> None:
        addr = _deploy(plain_w3, account_address)
        data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [42])
        tx_hash = w3.seismic.send_shielded_transaction(to=addr, data=data, eip712=True)  # type: ignore[attr-defined]
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["type"] == SEISMIC_TX_TYPE


class TestEIP712NamespaceSignedCall:
    """Low-level w3.seismic.signed_call with eip712=True."""

    def test_returns_result(
        self, w3: Web3, plain_w3: Web3, account_address: str
    ) -> None:
        addr = _deploy(plain_w3, account_address)

        # Set to 11 first (also via eip712)
        set_data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [11])
        tx_hash = w3.seismic.send_shielded_transaction(
            to=addr, data=set_data, eip712=True
        )  # type: ignore[attr-defined]
        w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)

        # Now signed_call isOdd with eip712
        call_data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "isOdd", [])
        result = w3.seismic.signed_call(to=addr, data=call_data, eip712=True)  # type: ignore[attr-defined]
        assert result is not None
        assert int.from_bytes(result[-32:], "big") == 1
