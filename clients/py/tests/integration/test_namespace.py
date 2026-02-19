"""Integration tests for w3.seismic namespace (low-level API)."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from web3 import Web3

from seismic_web3.chains import SEISMIC_TX_TYPE
from seismic_web3.contract.abi import encode_shielded_calldata
from tests.integration.contracts import (
    SEISMIC_COUNTER_ABI,
    SEISMIC_COUNTER_BYTECODE,
    deploy_contract,
)


def _deploy(plain_w3: Web3, account_address: str) -> ChecksumAddress:
    return deploy_contract(plain_w3, SEISMIC_COUNTER_BYTECODE, account_address)


class TestSendShieldedTransaction:
    def test_returns_hash(self, w3: Web3, plain_w3: Web3, account_address: str) -> None:
        addr = _deploy(plain_w3, account_address)
        data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [42])
        tx_hash = w3.seismic.send_shielded_transaction(to=addr, data=data)  # type: ignore[attr-defined]
        assert len(tx_hash) == 32

    def test_tx_type(self, w3: Web3, plain_w3: Web3, account_address: str) -> None:
        addr = _deploy(plain_w3, account_address)
        data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [42])
        tx_hash = w3.seismic.send_shielded_transaction(to=addr, data=data)  # type: ignore[attr-defined]
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["type"] == SEISMIC_TX_TYPE


class TestSignedCall:
    def test_returns_result(
        self, w3: Web3, plain_w3: Web3, account_address: str
    ) -> None:
        addr = _deploy(plain_w3, account_address)

        # Set to 11 first
        set_data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "setNumber", [11])
        tx_hash = w3.seismic.send_shielded_transaction(to=addr, data=set_data)  # type: ignore[attr-defined]
        w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)

        # Now signed_call isOdd
        call_data = encode_shielded_calldata(SEISMIC_COUNTER_ABI, "isOdd", [])
        result = w3.seismic.signed_call(to=addr, data=call_data)  # type: ignore[attr-defined]
        assert result is not None
        assert int.from_bytes(result[-32:], "big") == 1
