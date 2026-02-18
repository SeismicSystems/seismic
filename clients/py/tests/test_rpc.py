"""Tests for seismic_web3.rpc — custom Seismic JSON-RPC methods."""

from unittest.mock import AsyncMock, MagicMock

import pytest
from web3 import AsyncWeb3, Web3

from seismic_web3._types import CompressedPublicKey
from seismic_web3.rpc import async_get_tee_public_key, get_tee_public_key

# A valid 33-byte compressed public key (02 prefix + 32 bytes).
MOCK_TEE_PK_HEX = "0x02" + "ab" * 32
# Same key without the 0x prefix (some nodes return it this way).
MOCK_TEE_PK_NO_PREFIX = "02" + "ab" * 32


class TestGetTeePublicKeySync:
    def test_returns_compressed_public_key(self):
        w3 = MagicMock(spec=Web3)
        w3.provider.make_request.return_value = {"result": MOCK_TEE_PK_HEX}

        pk = get_tee_public_key(w3)

        assert isinstance(pk, CompressedPublicKey)
        assert len(pk) == 33
        assert pk[0] == 0x02
        w3.provider.make_request.assert_called_once_with("seismic_getTeePublicKey", [])

    def test_handles_no_0x_prefix(self):
        w3 = MagicMock(spec=Web3)
        w3.provider.make_request.return_value = {"result": MOCK_TEE_PK_NO_PREFIX}

        pk = get_tee_public_key(w3)

        assert isinstance(pk, CompressedPublicKey)
        assert len(pk) == 33

    def test_invalid_key_raises(self):
        w3 = MagicMock(spec=Web3)
        # Return a key that's too short.
        w3.provider.make_request.return_value = {"result": "0x02" + "ab" * 16}

        with pytest.raises(ValueError, match="expected 33 bytes"):
            get_tee_public_key(w3)


class TestGetTeePublicKeyAsync:
    @pytest.mark.asyncio
    async def test_returns_compressed_public_key(self):
        w3 = MagicMock(spec=AsyncWeb3)
        w3.provider.make_request = AsyncMock(return_value={"result": MOCK_TEE_PK_HEX})

        pk = await async_get_tee_public_key(w3)

        assert isinstance(pk, CompressedPublicKey)
        assert len(pk) == 33
        assert pk[0] == 0x02
        w3.provider.make_request.assert_awaited_once_with("seismic_getTeePublicKey", [])

    @pytest.mark.asyncio
    async def test_handles_no_0x_prefix(self):
        w3 = MagicMock(spec=AsyncWeb3)
        w3.provider.make_request = AsyncMock(
            return_value={"result": MOCK_TEE_PK_NO_PREFIX}
        )

        pk = await async_get_tee_public_key(w3)

        assert isinstance(pk, CompressedPublicKey)
        assert len(pk) == 33
