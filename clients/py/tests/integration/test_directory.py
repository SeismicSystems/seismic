"""Integration tests for Directory contract helpers.

Requires seismic-reth (``CHAIN=reth``) with the Directory genesis
contract at ``0x1000000000000000000000000000000000000004``.

Skipped when ``CHAIN != reth`` because sanvil does not currently
include the Directory / Intelligence genesis contracts in its
genesis block.  Once sanvil ships with these contracts, remove the
skip marker below.
"""

from __future__ import annotations

import os
from typing import TYPE_CHECKING

import pytest
from web3 import Web3

from seismic_web3._types import Bytes32, PrivateKey
from seismic_web3.src20.directory import (
    check_has_key,
    compute_key_hash,
    get_key_hash,
    get_viewing_key,
    register_viewing_key,
)

if TYPE_CHECKING:
    from seismic_web3.client import EncryptionState

# ---------------------------------------------------------------------------
# Skip the entire module unless running against seismic-reth, which is the
# only backend that currently deploys the Directory contract at its genesis
# address.  sanvil does not include genesis contracts yet.
# ---------------------------------------------------------------------------
pytestmark = pytest.mark.skipif(
    os.environ.get("CHAIN", "anvil") != "reth",
    reason=(
        "Directory genesis contract not available on sanvil — "
        "run with CHAIN=reth to enable these tests"
    ),
)


@pytest.fixture(autouse=True)
def _warmup_chain(w3: Web3, account_address: str) -> None:
    """Send a no-op tx so reth dev mode has a non-genesis recent block.

    seismic-reth's ``recent_block_hash`` validation fails when the only
    block is block 0 (genesis).  A single plain transfer creates block 1
    and unblocks subsequent shielded transactions.
    """
    if w3.eth.block_number == 0:
        tx = w3.eth.send_transaction(
            {"from": account_address, "to": account_address, "value": 0}
        )
        w3.eth.wait_for_transaction_receipt(tx, timeout=10)


@pytest.fixture
def viewing_key() -> Bytes32:
    """A deterministic 32-byte test viewing key."""
    return Bytes32(b"\xab" * 32)


class TestRegisterAndGetViewingKey:
    """Test registering and retrieving viewing keys from the Directory."""

    def test_register_and_get_viewing_key(
        self,
        w3: Web3,
        viewing_key: Bytes32,
        account_address: str,
    ) -> None:
        encryption: EncryptionState = w3.seismic.encryption  # type: ignore[attr-defined]
        private_key: PrivateKey = w3.seismic._private_key  # type: ignore[attr-defined]

        # Register the viewing key
        tx_hash = register_viewing_key(w3, encryption, private_key, viewing_key)
        assert len(tx_hash) == 32
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1

        # Retrieve it back via signed read
        result = get_viewing_key(w3, encryption, private_key)
        assert bytes(result) == bytes(viewing_key)


class TestCheckHasKey:
    """Test checking key registration status."""

    def test_check_has_key_after_register(
        self,
        w3: Web3,
        viewing_key: Bytes32,
        account_address: str,
    ) -> None:
        encryption: EncryptionState = w3.seismic.encryption  # type: ignore[attr-defined]
        private_key: PrivateKey = w3.seismic._private_key  # type: ignore[attr-defined]

        # Register a key first
        tx_hash = register_viewing_key(w3, encryption, private_key, viewing_key)
        w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)

        # Check that the account now has a key
        has_key = check_has_key(w3, Web3.to_checksum_address(account_address))
        assert has_key is True


class TestComputeKeyHash:
    """Test that local key hash matches on-chain key hash."""

    def test_compute_key_hash_matches_on_chain(
        self,
        w3: Web3,
        viewing_key: Bytes32,
        account_address: str,
    ) -> None:
        encryption: EncryptionState = w3.seismic.encryption  # type: ignore[attr-defined]
        private_key: PrivateKey = w3.seismic._private_key  # type: ignore[attr-defined]

        # Register a key first
        tx_hash = register_viewing_key(w3, encryption, private_key, viewing_key)
        w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)

        # Compare local hash with on-chain hash
        local_hash = compute_key_hash(viewing_key)
        on_chain_hash = get_key_hash(w3, Web3.to_checksum_address(account_address))
        assert local_hash == on_chain_hash
