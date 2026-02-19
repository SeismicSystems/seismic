"""Integration tests for SRC20 token (TestToken with shielded balances)."""

import pytest
from hexbytes import HexBytes
from web3 import Web3

from seismic_web3.contract.shielded import ShieldedContract
from tests.integration.contracts import (
    TEST_TOKEN_ABI,
    TEST_TOKEN_BYTECODE,
    deploy_contract,
)


def _decode_uint256(raw: HexBytes) -> int:
    """Decode a uint256 from raw ABI-encoded bytes."""
    return int.from_bytes(raw[-32:], "big")


def _decode_bool(raw: HexBytes) -> int:
    """Decode a bool from raw ABI-encoded bytes."""
    return int.from_bytes(raw[-32:], "big")


@pytest.fixture
def token(w3: Web3, plain_w3: Web3, account_address: str) -> ShieldedContract:
    """Deploy a fresh TestToken and return a ShieldedContract."""
    addr = deploy_contract(plain_w3, TEST_TOKEN_BYTECODE, account_address)
    return w3.seismic.contract(addr, TEST_TOKEN_ABI)  # type: ignore[attr-defined]


class TestTokenMetadata:
    """Test SRC20 metadata functions (name, symbol, decimals)."""

    def test_decimals(self, token: ShieldedContract) -> None:
        result = token.tread.decimals()
        assert _decode_uint256(result) == 18

    def test_name_is_nonempty(self, token: ShieldedContract) -> None:
        result = token.tread.name()
        assert result is not None
        assert len(result) > 0

    def test_symbol_is_nonempty(self, token: ShieldedContract) -> None:
        result = token.tread.symbol()
        assert result is not None
        assert len(result) > 0


class TestBalanceOf:
    """Verify balanceOf() works with no arguments (SRC20 vs ERC20 difference)."""

    def test_initial_balance_is_zero(self, token: ShieldedContract) -> None:
        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 0


class TestMint:
    """Test minting (admin-only, uses shielded amount)."""

    def test_mint_succeeds(
        self,
        token: ShieldedContract,
        w3: Web3,
        account_address: str,
    ) -> None:
        tx_hash = token.write.mint(account_address, 1000)
        assert len(tx_hash) == 32
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

    def test_balance_after_mint(
        self,
        token: ShieldedContract,
        w3: Web3,
        account_address: str,
    ) -> None:
        tx = token.write.mint(account_address, 500)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 500

    def test_mint_multiple_adds_up(
        self,
        token: ShieldedContract,
        w3: Web3,
        account_address: str,
    ) -> None:
        tx1 = token.write.mint(account_address, 300)
        w3.eth.wait_for_transaction_receipt(tx1, timeout=30)

        tx2 = token.write.mint(account_address, 200)
        w3.eth.wait_for_transaction_receipt(tx2, timeout=30)

        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 500


class TestTransfer:
    """Test transfer (shielded amount)."""

    def test_transfer_succeeds(
        self,
        token: ShieldedContract,
        w3: Web3,
        account_address: str,
    ) -> None:
        tx = token.write.mint(account_address, 1000)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        recipient = "0x000000000000000000000000000000000000dEaD"
        tx = token.write.transfer(recipient, 100)
        receipt = w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert receipt["status"] == 1

    def test_balance_decreases_after_transfer(
        self,
        token: ShieldedContract,
        w3: Web3,
        account_address: str,
    ) -> None:
        tx = token.write.mint(account_address, 1000)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        recipient = "0x000000000000000000000000000000000000dEaD"
        tx = token.write.transfer(recipient, 400)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 600


class TestApprove:
    """Test approve (shielded amount)."""

    def test_approve_succeeds(self, token: ShieldedContract, w3: Web3) -> None:
        spender = "0x000000000000000000000000000000000000dEaD"
        tx = token.write.approve(spender, 500)
        receipt = w3.eth.wait_for_transaction_receipt(tx, timeout=30)
        assert receipt["status"] == 1


class TestBurn:
    """Test burn (admin-only)."""

    def test_burn_reduces_balance(
        self,
        token: ShieldedContract,
        w3: Web3,
        account_address: str,
    ) -> None:
        tx = token.write.mint(account_address, 1000)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        tx = token.write.burn(account_address, 300)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 700


class TestSRC20Lifecycle:
    """End-to-end lifecycle: mint -> transfer -> burn -> check balances."""

    def test_full_lifecycle(
        self,
        token: ShieldedContract,
        w3: Web3,
        account_address: str,
    ) -> None:
        # 1. Initial balance is 0
        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 0

        # 2. Mint 1000 to self
        tx = token.write.mint(account_address, 1000)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        # 3. Balance is now 1000
        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 1000

        # 4. Transfer 250 to another address
        recipient = "0x000000000000000000000000000000000000dEaD"
        tx = token.write.transfer(recipient, 250)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        # 5. Balance is now 750
        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 750

        # 6. Burn 150 from self
        tx = token.write.burn(account_address, 150)
        w3.eth.wait_for_transaction_receipt(tx, timeout=30)

        # 7. Balance is now 600
        result = token.read.balanceOf()
        assert result is not None
        assert _decode_uint256(result) == 600
