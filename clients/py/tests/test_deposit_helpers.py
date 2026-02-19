"""Unit tests for deposit contract helper functions."""

from __future__ import annotations

import pytest

from seismic_web3.abis.deposit_contract import (
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
    compute_deposit_data_root,
    make_withdrawal_credentials,
)

# ---------------------------------------------------------------------------
# Test data — matches DepositContract.t.sol constants
# ---------------------------------------------------------------------------

NODE_PUBKEY = bytes.fromhex(
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
)  # 32 bytes

CONSENSUS_PUBKEY = bytes.fromhex(
    "1234567890abcdef1234567890abcdef1234567890abcdef"
    "1234567890abcdef1234567890abcdef1234567890abcdef"
)  # 48 bytes

WITHDRAWAL_CREDENTIALS = bytes.fromhex(
    "0100000000000000000000001234567890abcdef1234567890abcdef12345678"
)  # 32 bytes

NODE_SIGNATURE = bytes.fromhex(
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
)  # 64 bytes

CONSENSUS_SIGNATURE = bytes.fromhex(
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
)  # 96 bytes


# ---------------------------------------------------------------------------
# make_withdrawal_credentials
# ---------------------------------------------------------------------------


_TEST_ADDR = "0x1234567890abcdef1234567890abcdef12345678"


class TestMakeWithdrawalCredentials:
    def test_format(self) -> None:
        creds = make_withdrawal_credentials(_TEST_ADDR)
        assert len(creds) == 32
        assert creds[0:1] == b"\x01"
        assert creds[1:12] == b"\x00" * 11
        assert creds[12:] == bytes.fromhex(_TEST_ADDR[2:])

    def test_without_0x_prefix(self) -> None:
        creds = make_withdrawal_credentials(_TEST_ADDR[2:])
        assert len(creds) == 32
        assert creds[0:1] == b"\x01"

    def test_matches_solidity_test_vector(self) -> None:
        """Solidity test uses this exact withdrawal credentials."""
        creds = make_withdrawal_credentials(_TEST_ADDR)
        assert creds == WITHDRAWAL_CREDENTIALS

    def test_rejects_short_address(self) -> None:
        with pytest.raises(ValueError, match="20 bytes"):
            make_withdrawal_credentials("0x1234")

    def test_rejects_long_address(self) -> None:
        with pytest.raises(ValueError, match="20 bytes"):
            make_withdrawal_credentials("0x" + "ab" * 21)


# ---------------------------------------------------------------------------
# compute_deposit_data_root
# ---------------------------------------------------------------------------


class TestComputeDepositDataRoot:
    def test_deterministic(self) -> None:
        """Same inputs always produce the same root."""
        amount_gwei = 32_000_000_000  # 32 ETH
        root1 = compute_deposit_data_root(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            amount_gwei,
        )
        root2 = compute_deposit_data_root(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            amount_gwei,
        )
        assert root1 == root2

    def test_returns_32_bytes(self) -> None:
        root = compute_deposit_data_root(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            32_000_000_000,
        )
        assert len(root) == 32

    def test_different_amount_gives_different_root(self) -> None:
        root_32 = compute_deposit_data_root(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            32_000_000_000,
        )
        root_1 = compute_deposit_data_root(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            1_000_000_000,
        )
        assert root_32 != root_1


# ---------------------------------------------------------------------------
# ABI and address constants
# ---------------------------------------------------------------------------


class TestConstants:
    def test_abi_has_deposit_function(self) -> None:
        fn_names = [e["name"] for e in DEPOSIT_CONTRACT_ABI if e["type"] == "function"]
        assert "deposit" in fn_names

    def test_abi_has_view_functions(self) -> None:
        fn_names = [e["name"] for e in DEPOSIT_CONTRACT_ABI if e["type"] == "function"]
        assert "get_deposit_root" in fn_names
        assert "get_deposit_count" in fn_names
        assert "supportsInterface" in fn_names

    def test_abi_has_event(self) -> None:
        events = [e for e in DEPOSIT_CONTRACT_ABI if e["type"] == "event"]
        assert len(events) == 1
        assert events[0]["name"] == "DepositEvent"

    def test_deposit_contract_address(self) -> None:
        assert DEPOSIT_CONTRACT_ADDRESS.startswith("0x")
        assert len(DEPOSIT_CONTRACT_ADDRESS) == 42
