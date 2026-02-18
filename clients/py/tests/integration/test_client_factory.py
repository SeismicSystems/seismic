"""Integration tests for create_shielded_web3 / create_async_shielded_web3."""

import pytest
from web3 import AsyncWeb3, Web3

from seismic_web3 import CompressedPublicKey


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
