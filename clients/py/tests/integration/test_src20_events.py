"""Integration tests for SRC20 event watching with viewing keys.

Uses the MockSRC20Events contract to emit Transfer and Approval
events with pre-encrypted amounts, then verifies the watcher
decrypts them correctly.
"""

from __future__ import annotations

import time
from typing import TYPE_CHECKING

import pytest
from eth_hash.auto import keccak
from hexbytes import HexBytes
from web3 import Web3

from seismic_web3._types import Bytes32, EncryptionNonce
from seismic_web3.crypto.aes import AesGcmCrypto
from seismic_web3.src20.crypto import decrypt_encrypted_amount, parse_encrypted_data
from seismic_web3.src20.directory import compute_key_hash
from seismic_web3.src20.watch import (
    SRC20EventWatcher,
    watch_src20_events_with_key,
)

if TYPE_CHECKING:
    from seismic_web3.src20.types import DecryptedApprovalLog, DecryptedTransferLog
from tests.integration.contracts import (
    MOCK_SRC20_EVENTS_ABI,
    MOCK_SRC20_EVENTS_BYTECODE,
    deploy_contract,
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

#: Deterministic test viewing key
TEST_KEY = Bytes32(b"\xab" * 32)

#: Deterministic 12-byte nonce for test encryption
TEST_NONCE = EncryptionNonce(b"\x01" * 12)


def _encrypt_amount(key: Bytes32, amount: int, nonce: EncryptionNonce) -> bytes:
    """Encrypt a uint256 amount using AES-256-GCM and pack as ciphertext||nonce."""
    plaintext = HexBytes(amount.to_bytes(32, "big"))
    crypto = AesGcmCrypto(key)
    ct = crypto.encrypt(plaintext, nonce, aad=None)
    return bytes(ct) + bytes(nonce)


# ---------------------------------------------------------------------------
# Unit-level tests for crypto helpers (no chain needed)
# ---------------------------------------------------------------------------


class TestCryptoHelpers:
    """Test parse_encrypted_data and decrypt_encrypted_amount."""

    def test_parse_encrypted_data(self) -> None:
        nonce = b"\x01" * 12
        ciphertext = b"\xff" * 48  # 32 bytes ct + 16 bytes tag
        packed = ciphertext + nonce
        ct, n = parse_encrypted_data(packed)
        assert ct == ciphertext
        assert n == nonce

    def test_parse_empty_raises(self) -> None:
        with pytest.raises(ValueError, match="empty or too short"):
            parse_encrypted_data(b"")

    def test_decrypt_encrypted_amount_roundtrip(self) -> None:
        amount = 42_000
        encrypted = _encrypt_amount(TEST_KEY, amount, TEST_NONCE)
        decrypted = decrypt_encrypted_amount(TEST_KEY, encrypted)
        assert decrypted == amount

    def test_compute_key_hash(self) -> None:
        key = Bytes32(b"\xaa" * 32)
        result = compute_key_hash(key)
        expected = keccak(bytes(key))
        assert result == expected


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def mock_events_address(plain_w3: Web3, account_address: str) -> str:
    """Deploy MockSRC20Events and return its address."""
    return deploy_contract(plain_w3, MOCK_SRC20_EVENTS_BYTECODE, account_address)


# ---------------------------------------------------------------------------
# Integration tests
# ---------------------------------------------------------------------------


class TestWatchTransferEvent:
    """Test watching Transfer events with encrypted amounts."""

    def test_watch_transfer_event_with_key(
        self,
        w3: Web3,
        plain_w3: Web3,
        mock_events_address: str,
        account_address: str,
    ) -> None:
        """Emit a Transfer event with encrypted data, watch and decrypt it."""
        amount = 1_000
        encrypted = _encrypt_amount(TEST_KEY, amount, TEST_NONCE)
        key_hash = HexBytes(compute_key_hash(TEST_KEY))

        # Record the block before emitting
        start_block = w3.eth.block_number

        # Emit the Transfer event via the mock contract
        from_addr = account_address
        to_addr = "0x000000000000000000000000000000000000dEaD"

        contract = plain_w3.eth.contract(
            address=Web3.to_checksum_address(mock_events_address),
            abi=MOCK_SRC20_EVENTS_ABI,
        )
        tx_hash = contract.functions.emitTransfer(
            from_addr,
            Web3.to_checksum_address(to_addr),
            key_hash,
            encrypted,
        ).transact({"from": account_address})
        receipt = plain_w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

        # Set up watcher and collect results
        received: list[DecryptedTransferLog] = []
        errors: list[Exception] = []

        watcher = watch_src20_events_with_key(
            w3,
            viewing_key=TEST_KEY,
            token_address=Web3.to_checksum_address(mock_events_address),
            on_transfer=lambda log: received.append(log),
            on_error=lambda err: errors.append(err),
            poll_interval=0.5,
            from_block=start_block,
        )

        try:
            # Give it time to poll
            deadline = time.monotonic() + 10
            while not received and time.monotonic() < deadline:
                time.sleep(0.3)
        finally:
            watcher.stop()

        assert len(errors) == 0, f"Unexpected errors: {errors}"
        assert len(received) == 1

        log = received[0]
        assert log.decrypted_amount == amount
        assert log.from_address == Web3.to_checksum_address(from_addr)
        assert log.to_address == Web3.to_checksum_address(to_addr)
        assert log.encrypt_key_hash == bytes(key_hash)


class TestWatchApprovalEvent:
    """Test watching Approval events with encrypted amounts."""

    def test_watch_approval_event_with_key(
        self,
        w3: Web3,
        plain_w3: Web3,
        mock_events_address: str,
        account_address: str,
    ) -> None:
        amount = 500
        encrypted = _encrypt_amount(TEST_KEY, amount, TEST_NONCE)
        key_hash = HexBytes(compute_key_hash(TEST_KEY))

        start_block = w3.eth.block_number

        owner = account_address
        spender = "0x000000000000000000000000000000000000dEaD"

        contract = plain_w3.eth.contract(
            address=Web3.to_checksum_address(mock_events_address),
            abi=MOCK_SRC20_EVENTS_ABI,
        )
        tx_hash = contract.functions.emitApproval(
            owner,
            Web3.to_checksum_address(spender),
            key_hash,
            encrypted,
        ).transact({"from": account_address})
        receipt = plain_w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

        received: list[DecryptedApprovalLog] = []
        errors: list[Exception] = []

        watcher = watch_src20_events_with_key(
            w3,
            viewing_key=TEST_KEY,
            token_address=Web3.to_checksum_address(mock_events_address),
            on_approval=lambda log: received.append(log),
            on_error=lambda err: errors.append(err),
            poll_interval=0.5,
            from_block=start_block,
        )

        try:
            deadline = time.monotonic() + 10
            while not received and time.monotonic() < deadline:
                time.sleep(0.3)
        finally:
            watcher.stop()

        assert len(errors) == 0, f"Unexpected errors: {errors}"
        assert len(received) == 1

        log = received[0]
        assert log.decrypted_amount == amount
        assert log.owner == Web3.to_checksum_address(owner)
        assert log.spender == Web3.to_checksum_address(spender)


class TestWatcherLifecycle:
    """Test watcher start/stop behavior."""

    def test_watcher_stop(self, w3: Web3) -> None:
        """Verify .stop() cleanly terminates the polling thread."""
        watcher = watch_src20_events_with_key(
            w3,
            viewing_key=TEST_KEY,
            poll_interval=0.2,
        )
        assert watcher.is_running

        watcher.stop()
        assert not watcher.is_running

    def test_context_manager(self, w3: Web3) -> None:
        """Verify context manager usage starts and stops the watcher."""
        watcher = SRC20EventWatcher(
            w3,
            TEST_KEY,
            poll_interval=0.2,
        )
        assert not watcher.is_running

        with watcher:
            assert watcher.is_running
            time.sleep(0.1)

        assert not watcher.is_running
