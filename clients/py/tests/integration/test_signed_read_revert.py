"""Regression tests for signed-read revert confidentiality.

A contract's revert data can embed private state (e.g. a custom error like
``revert InsufficientBalance(actualBalance)``), just like a successful return
value can. The node therefore encrypts the revert output of a signed read
under the caller's key instead of returning it in cleartext, and the client
decrypts it so revert reasons still surface to the signer.

The RevertLeak test contract stores a private ``suint256 secret`` and has
``revertWithSecret()`` which does ``revert("secret=<decimal value>")``.

Mirrors the seismic-viem tests in ``signedReadRevert.ts`` and the node-side
e2e test ``test_eth_call_signed_read_revert_leaks_private_data``.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from hexbytes import HexBytes
from web3.exceptions import ContractLogicError

from tests.integration.contracts import _load_artifact, deploy_contract

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from web3 import Web3

    from seismic_web3.contract.shielded import ShieldedContract

_revert_leak = _load_artifact("revert_leak.json")

REVERT_LEAK_ABI = _revert_leak["abi"]
REVERT_LEAK_BYTECODE = _revert_leak["bytecode"]

SECRET = 987654321


@pytest.fixture
def contract(
    w3: Web3, plain_w3: Web3, account_address: ChecksumAddress
) -> ShieldedContract:
    """Deploy RevertLeak and set its private secret via a shielded write."""
    addr = deploy_contract(plain_w3, REVERT_LEAK_BYTECODE, account_address)
    contract = w3.seismic.contract(addr, REVERT_LEAK_ABI)  # type: ignore[attr-defined]
    tx_hash = contract.swrite.setSecret(SECRET)
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
    assert receipt["status"] == 1
    return contract


class TestSignedReadRevert:
    def test_revert_reason_surfaces_without_wire_leak(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """A signed read that reverts raises ContractLogicError with the
        decrypted revert reason; the raw RPC error never carried it in
        cleartext (the node returns ciphertext, decrypted client-side)."""
        with pytest.raises(ContractLogicError) as excinfo:
            contract.sread.revertWithSecret()

        assert f"secret={SECRET}" in str(excinfo.value)

    def test_revert_data_is_plaintext_revert_encoding(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """The exception's data field carries the decrypted Error(string)
        revert data for ABI-aware decoding."""
        with pytest.raises(ContractLogicError) as excinfo:
            contract.sread.revertWithSecret()

        data = excinfo.value.data
        assert isinstance(data, str)
        assert data.startswith("0x")
        raw = HexBytes(data)
        # Error(string) selector
        assert raw[:4] == bytes.fromhex("08c379a0")
        assert f"secret={SECRET}".encode() in bytes(raw)

    def test_estimate_gas_revert_surfaces_reason(
        self, contract: ShieldedContract, w3: Web3
    ) -> None:
        """A shielded write without explicit gas routes through signed
        eth_estimateGas; on revert the decrypted reason surfaces the same
        way."""
        with pytest.raises(ContractLogicError) as excinfo:
            contract.swrite.revertWithSecret()

        assert f"secret={SECRET}" in str(excinfo.value)
