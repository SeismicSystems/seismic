"""Integration tests for the DepositContract (Eth2 validator staking)."""

import pytest
from hexbytes import HexBytes
from web3 import Web3

from seismic_web3.abis.deposit_contract import compute_deposit_data_root
from seismic_web3.contract.shielded import ShieldedContract
from tests.integration.contracts import (
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_BYTECODE,
    deploy_contract,
)

# ---------------------------------------------------------------------------
# Test data — same constants as Solidity tests (DepositContract.t.sol)
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
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def deposit_contract(
    w3: Web3, plain_w3: Web3, account_address: str
) -> ShieldedContract:
    """Deploy a fresh DepositContract and return a ShieldedContract wrapper."""
    addr = deploy_contract(plain_w3, DEPOSIT_CONTRACT_BYTECODE, account_address)
    return w3.seismic.contract(addr, DEPOSIT_CONTRACT_ABI)  # type: ignore[attr-defined]


def _parse_deposit_count(raw: bytes) -> int:
    """Parse the decoded get_deposit_count() bytes to an int (little-endian)."""
    return int.from_bytes(raw[:8], "little")


def _make_deposit(
    deposit_contract: ShieldedContract,
    w3: Web3,
    amount_ether: int = 32,
) -> HexBytes:
    """Helper: compute root and submit a deposit, return tx hash."""
    amount_gwei = amount_ether * 1_000_000_000
    deposit_data_root = compute_deposit_data_root(
        node_pubkey=NODE_PUBKEY,
        consensus_pubkey=CONSENSUS_PUBKEY,
        withdrawal_credentials=WITHDRAWAL_CREDENTIALS,
        node_signature=NODE_SIGNATURE,
        consensus_signature=CONSENSUS_SIGNATURE,
        amount_gwei=amount_gwei,
    )
    tx_hash = deposit_contract.twrite.deposit(
        NODE_PUBKEY,
        CONSENSUS_PUBKEY,
        WITHDRAWAL_CREDENTIALS,
        NODE_SIGNATURE,
        CONSENSUS_SIGNATURE,
        deposit_data_root,
        value=amount_ether * 10**18,
    )
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
    assert receipt["status"] == 1, "Deposit transaction failed"
    return tx_hash


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestDepositContractReads:
    """Test view functions on a fresh (empty) contract."""

    def test_initial_deposit_count_is_zero(
        self, deposit_contract: ShieldedContract
    ) -> None:
        raw = deposit_contract.tread.get_deposit_count()
        assert _parse_deposit_count(raw) == 0

    def test_initial_deposit_root_is_nonempty(
        self, deposit_contract: ShieldedContract
    ) -> None:
        root = deposit_contract.tread.get_deposit_root()
        assert root is not None
        assert len(root) == 32

    def test_supports_interface(self, deposit_contract: ShieldedContract) -> None:
        # ERC165 interface ID
        erc165_id = bytes.fromhex("01ffc9a7")
        assert deposit_contract.tread.supportsInterface(erc165_id) is True


class TestDeposit:
    """Test successful deposits."""

    def test_deposit_32_eth(
        self,
        deposit_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        tx_hash = _make_deposit(deposit_contract, w3, 32)
        assert len(tx_hash) == 32

    def test_deposit_increments_count(
        self,
        deposit_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        _make_deposit(deposit_contract, w3, 32)
        raw = deposit_contract.tread.get_deposit_count()
        assert _parse_deposit_count(raw) == 1

    def test_deposit_changes_root(
        self,
        deposit_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        root_before = deposit_contract.tread.get_deposit_root()
        _make_deposit(deposit_contract, w3, 32)
        root_after = deposit_contract.tread.get_deposit_root()
        assert root_before != root_after


class TestMinimumDeposit:
    """Test minimum deposit amount (1 ETH)."""

    def test_1_eth_deposit(
        self,
        deposit_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        tx_hash = _make_deposit(deposit_contract, w3, 1)
        assert len(tx_hash) == 32

        raw = deposit_contract.tread.get_deposit_count()
        assert _parse_deposit_count(raw) == 1


class TestMultipleDeposits:
    """Test multiple deposits increment the count correctly."""

    def test_two_deposits(
        self,
        deposit_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        _make_deposit(deposit_contract, w3, 32)
        _make_deposit(deposit_contract, w3, 32)

        raw = deposit_contract.tread.get_deposit_count()
        assert _parse_deposit_count(raw) == 2


class TestDepositLifecycle:
    """End-to-end lifecycle: check initial state, deposit, verify changes."""

    def test_full_lifecycle(
        self,
        deposit_contract: ShieldedContract,
        w3: Web3,
    ) -> None:
        # 1. Initial count is 0
        raw = deposit_contract.tread.get_deposit_count()
        assert _parse_deposit_count(raw) == 0

        # 2. Initial root is some value
        initial_root = deposit_contract.tread.get_deposit_root()
        assert len(initial_root) == 32

        # 3. Make a deposit
        _make_deposit(deposit_contract, w3, 32)

        # 4. Count is now 1
        raw = deposit_contract.tread.get_deposit_count()
        assert _parse_deposit_count(raw) == 1

        # 5. Root changed
        new_root = deposit_contract.tread.get_deposit_root()
        assert new_root != initial_root

        # 6. Second deposit
        _make_deposit(deposit_contract, w3, 32)

        # 7. Count is now 2
        raw = deposit_contract.tread.get_deposit_count()
        assert _parse_deposit_count(raw) == 2

        # 8. Root changed again
        final_root = deposit_contract.tread.get_deposit_root()
        assert final_root != new_root


# ---------------------------------------------------------------------------
# Tests — deposit action sugar (w3.seismic.*)
# ---------------------------------------------------------------------------


@pytest.fixture
def deposit_address(
    plain_w3: Web3,
    account_address: str,
) -> str:
    """Deploy a fresh DepositContract and return its address."""
    return deploy_contract(
        plain_w3,
        DEPOSIT_CONTRACT_BYTECODE,
        account_address,
    )


class TestDepositActions:
    """Test the syntactic sugar methods on ``w3.seismic``."""

    def test_get_deposit_count_initial(
        self,
        w3: Web3,
        deposit_address: str,
    ) -> None:
        count = w3.seismic.get_deposit_count(address=deposit_address)
        assert count == 0

    def test_get_deposit_root_initial(
        self,
        w3: Web3,
        deposit_address: str,
    ) -> None:
        root = w3.seismic.get_deposit_root(address=deposit_address)
        assert isinstance(root, bytes)
        assert len(root) == 32

    def test_deposit_and_count(
        self,
        w3: Web3,
        deposit_address: str,
    ) -> None:
        amount_gwei = 32_000_000_000
        deposit_data_root = compute_deposit_data_root(
            node_pubkey=NODE_PUBKEY,
            consensus_pubkey=CONSENSUS_PUBKEY,
            withdrawal_credentials=WITHDRAWAL_CREDENTIALS,
            node_signature=NODE_SIGNATURE,
            consensus_signature=CONSENSUS_SIGNATURE,
            amount_gwei=amount_gwei,
        )
        tx_hash = w3.seismic.deposit(
            node_pubkey=NODE_PUBKEY,
            consensus_pubkey=CONSENSUS_PUBKEY,
            withdrawal_credentials=WITHDRAWAL_CREDENTIALS,
            node_signature=NODE_SIGNATURE,
            consensus_signature=CONSENSUS_SIGNATURE,
            deposit_data_root=deposit_data_root,
            value=32 * 10**18,
            address=deposit_address,
        )
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

        count = w3.seismic.get_deposit_count(address=deposit_address)
        assert count == 1

    def test_deposit_changes_root(
        self,
        w3: Web3,
        deposit_address: str,
    ) -> None:
        root_before = w3.seismic.get_deposit_root(
            address=deposit_address,
        )

        amount_gwei = 32_000_000_000
        deposit_data_root = compute_deposit_data_root(
            node_pubkey=NODE_PUBKEY,
            consensus_pubkey=CONSENSUS_PUBKEY,
            withdrawal_credentials=WITHDRAWAL_CREDENTIALS,
            node_signature=NODE_SIGNATURE,
            consensus_signature=CONSENSUS_SIGNATURE,
            amount_gwei=amount_gwei,
        )
        tx_hash = w3.seismic.deposit(
            node_pubkey=NODE_PUBKEY,
            consensus_pubkey=CONSENSUS_PUBKEY,
            withdrawal_credentials=WITHDRAWAL_CREDENTIALS,
            node_signature=NODE_SIGNATURE,
            consensus_signature=CONSENSUS_SIGNATURE,
            deposit_data_root=deposit_data_root,
            value=32 * 10**18,
            address=deposit_address,
        )
        w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)

        root_after = w3.seismic.get_deposit_root(
            address=deposit_address,
        )
        assert root_before != root_after
