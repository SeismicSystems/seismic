"""Integration tests for wallet and public client factories."""

import pytest
from web3 import AsyncWeb3, Web3

from seismic_web3 import CompressedPublicKey, PrivateKey
from seismic_web3.client import create_async_wallet_client, create_wallet_client
from seismic_web3.module import SeismicPublicNamespace
from tests.integration.contracts import (
    SEISMIC_COUNTER_ABI,
    SEISMIC_COUNTER_BYTECODE,
    TRANSPARENT_COUNTER_ABI,
    TRANSPARENT_COUNTER_BYTECODE,
    deploy_contract,
)


class TestSyncFactory:
    def test_creates_web3_with_seismic_namespace(self, w3: Web3) -> None:
        assert hasattr(w3, "seismic")

    def test_get_tee_public_key(self, w3: Web3) -> None:
        pk = w3.seismic.get_tee_public_key()  # type: ignore[attr-defined]
        assert isinstance(pk, CompressedPublicKey)
        assert len(pk) == 33

    def test_encryption_state_initialized(self, w3: Web3) -> None:
        assert len(w3.seismic.encryption.aes_key) == 32  # type: ignore[attr-defined]

    def test_chain_id(self, w3: Web3, expected_chain_id: int) -> None:
        assert w3.eth.chain_id == expected_chain_id


class TestAsyncFactory:
    @pytest.mark.asyncio
    async def test_async_creates_web3(self, async_w3: AsyncWeb3) -> None:
        assert hasattr(async_w3, "seismic")

    @pytest.mark.asyncio
    async def test_async_get_tee_public_key(self, async_w3: AsyncWeb3) -> None:
        pk = await async_w3.seismic.get_tee_public_key()  # type: ignore[attr-defined]
        assert isinstance(pk, CompressedPublicKey)
        assert len(pk) == 33


class TestPublicSyncFactory:
    def test_creates_public_namespace(self, public_w3: Web3) -> None:
        assert hasattr(public_w3, "seismic")
        assert isinstance(public_w3.seismic, SeismicPublicNamespace)  # type: ignore[attr-defined]

    def test_get_tee_public_key(self, public_w3: Web3) -> None:
        pk = public_w3.seismic.get_tee_public_key()  # type: ignore[attr-defined]
        assert isinstance(pk, CompressedPublicKey)
        assert len(pk) == 33

    def test_no_encryption_state(self, public_w3: Web3) -> None:
        assert not hasattr(public_w3.seismic, "encryption")  # type: ignore[attr-defined]

    def test_chain_id(self, public_w3: Web3, expected_chain_id: int) -> None:
        assert public_w3.eth.chain_id == expected_chain_id


class TestWalletClientSigning:
    """A wallet client must be able to send transparent txs as created.

    Seismic nodes keep no unlocked accounts, so ``eth_sendTransaction``
    only works if the factory installs local signing itself.
    """

    def test_fresh_client_can_twrite(
        self, rpc_url: str, private_key: PrivateKey, account_address: str
    ) -> None:
        fresh = create_wallet_client(rpc_url, private_key=private_key)
        assert fresh.eth.default_account == account_address

        addr = deploy_contract(fresh, TRANSPARENT_COUNTER_BYTECODE, account_address)
        contract = fresh.seismic.contract(addr, TRANSPARENT_COUNTER_ABI)  # type: ignore[attr-defined]
        tx_hash = contract.twrite.setNumber(5)
        receipt = fresh.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert contract.tread.number() == 5

    @pytest.mark.asyncio
    async def test_fresh_async_client_can_send(
        self, rpc_url: str, private_key: PrivateKey, account_address: str
    ) -> None:
        fresh = await create_async_wallet_client(rpc_url, private_key=private_key)
        assert fresh.eth.default_account == account_address

        tx_hash = await fresh.eth.send_transaction(
            {"to": account_address, "value": 1},
        )
        receipt = await fresh.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1


class TestShieldedWriteNonces:
    """Shielded writes sent back to back must not reuse a nonce."""

    def test_back_to_back_writes(
        self, w3: Web3, plain_w3: Web3, account_address: str
    ) -> None:
        addr = deploy_contract(plain_w3, SEISMIC_COUNTER_BYTECODE, account_address)
        contract = w3.seismic.contract(addr, SEISMIC_COUNTER_ABI)  # type: ignore[attr-defined]

        hashes = [contract.write.setNumber(i) for i in range(3)]
        for tx_hash in hashes:
            receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
            assert receipt["status"] == 1
        assert contract.read.getNumber() == 2

    def test_explicit_nonce(
        self, w3: Web3, plain_w3: Web3, account_address: str
    ) -> None:
        addr = deploy_contract(plain_w3, SEISMIC_COUNTER_BYTECODE, account_address)
        contract = w3.seismic.contract(addr, SEISMIC_COUNTER_ABI)  # type: ignore[attr-defined]

        nonce = w3.eth.get_transaction_count(account_address, "pending")
        tx_hash = contract.write.setNumber(7, nonce=nonce)
        receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
        assert receipt["status"] == 1
        assert w3.eth.get_transaction(tx_hash)["nonce"] == nonce
