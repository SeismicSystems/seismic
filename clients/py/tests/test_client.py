"""Tests for seismic_web3.client — EncryptionState and get_encryption."""

from unittest.mock import MagicMock, patch

from hexbytes import HexBytes

from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)
from seismic_web3.chains import SANVIL, ChainConfig
from seismic_web3.client import EncryptionState, create_shielded_web3, get_encryption
from seismic_web3.crypto.secp import private_key_to_compressed_public_key
from seismic_web3.transaction_types import (
    LegacyFields,
    SeismicElements,
    TxSeismicMetadata,
)

# Test vector from seismic-viem — same keys used in test_crypto.py
_NETWORK_PK = CompressedPublicKey(
    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
)
_CLIENT_SK = PrivateKey(
    "0xa30363336e1bb949185292a2a302de86e447d98f3a43d823c8c234d9e3e5ad77"
)
_EXPECTED_AES_KEY = Bytes32(
    "0xbf0dd6556618d1bf8d1602bf80be3a0f7cc729973829bb9acb75bd77770d5b90"
)


class TestGetEncryption:
    def test_deterministic_with_known_keys(self):
        """get_encryption with a fixed client key produces the expected AES key."""
        state = get_encryption(_NETWORK_PK, _CLIENT_SK)
        assert state.aes_key == _EXPECTED_AES_KEY

    def test_returns_encryption_state(self):
        state = get_encryption(_NETWORK_PK, _CLIENT_SK)
        assert isinstance(state, EncryptionState)
        assert isinstance(state.encryption_pubkey, CompressedPublicKey)
        assert isinstance(state.encryption_private_key, PrivateKey)

    def test_random_key_when_none(self):
        """Without a client key, a random one is generated."""
        state1 = get_encryption(_NETWORK_PK)
        state2 = get_encryption(_NETWORK_PK)
        # Two calls with random keys should produce different AES keys
        assert state1.aes_key != state2.aes_key

    def test_pubkey_matches_private_key(self):
        """The encryption_pubkey should be derived from encryption_private_key."""
        state = get_encryption(_NETWORK_PK, _CLIENT_SK)
        expected = private_key_to_compressed_public_key(_CLIENT_SK)
        assert state.encryption_pubkey == expected


class TestEncryptionState:
    def _make_metadata(self) -> TxSeismicMetadata:
        """Create a minimal metadata for testing."""
        return TxSeismicMetadata(
            sender="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            legacy_fields=LegacyFields(
                chain_id=31337,
                nonce=0,
                to="0xd3e8763675e4c425df46cc3b5c0f6cbdac396046",
                value=0,
            ),
            seismic_elements=SeismicElements(
                encryption_pubkey=CompressedPublicKey(
                    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
                ),
                encryption_nonce=EncryptionNonce("0x46a2b6020bba77fcb1e676a6"),
                message_version=0,
                recent_block_hash=Bytes32(
                    "0x934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f90"
                ),
                expires_at_block=100,
                signed_read=False,
            ),
        )

    def test_encrypt_decrypt_round_trip(self):
        """Encrypting then decrypting should return the original plaintext."""
        state = get_encryption(_NETWORK_PK, _CLIENT_SK)
        metadata = self._make_metadata()
        nonce = EncryptionNonce("0x46a2b6020bba77fcb1e676a6")
        plaintext = HexBytes(b"Hello Seismic!")

        ciphertext = state.encrypt(plaintext, nonce, metadata)
        assert ciphertext != plaintext
        assert len(ciphertext) > len(plaintext)  # includes auth tag

        decrypted = state.decrypt(ciphertext, nonce, metadata)
        assert bytes(decrypted) == bytes(plaintext)

    def test_encrypt_empty_data(self):
        """Encrypting empty data returns empty data."""
        state = get_encryption(_NETWORK_PK, _CLIENT_SK)
        metadata = self._make_metadata()
        nonce = EncryptionNonce("0x46a2b6020bba77fcb1e676a6")

        ciphertext = state.encrypt(HexBytes(b""), nonce, metadata)
        assert bytes(ciphertext) == b""


_MOCK_TEE_PK = CompressedPublicKey(
    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
)
_TEST_PK = PrivateKey(b"\x01" * 32)


class TestCreateShieldedWeb3:
    """Factory function accepts a URL string."""

    @patch("seismic_web3.client.get_tee_public_key", return_value=_MOCK_TEE_PK)
    @patch("seismic_web3.client.Web3")
    def test_accepts_string(self, mock_web3, mock_get_tee):
        mock_web3.return_value = MagicMock()
        mock_web3.HTTPProvider = MagicMock()

        create_shielded_web3("http://localhost:8545", private_key=_TEST_PK)

        mock_web3.HTTPProvider.assert_called_once_with("http://localhost:8545")


class TestChainConfigCreateClient:
    """ChainConfig.create_client delegates to create_shielded_web3."""

    @patch("seismic_web3.client.get_tee_public_key", return_value=_MOCK_TEE_PK)
    @patch("seismic_web3.client.Web3")
    def test_create_client_uses_rpc_url(self, mock_web3, mock_get_tee):
        mock_web3.return_value = MagicMock()
        mock_web3.HTTPProvider = MagicMock()

        cfg = ChainConfig(chain_id=1, rpc_url="http://test:8545")
        cfg.create_client(_TEST_PK)

        mock_web3.HTTPProvider.assert_called_once_with("http://test:8545")

    @patch("seismic_web3.client.get_tee_public_key", return_value=_MOCK_TEE_PK)
    @patch("seismic_web3.client.Web3")
    def test_create_client_with_predefined_chain(self, mock_web3, mock_get_tee):
        mock_web3.return_value = MagicMock()
        mock_web3.HTTPProvider = MagicMock()

        SANVIL.create_client(_TEST_PK)

        mock_web3.HTTPProvider.assert_called_once_with("http://127.0.0.1:8545")


class TestChainConfigCreateAsyncClient:
    """ChainConfig.create_async_client delegates to create_async_shielded_web3."""

    @patch("seismic_web3.client.async_get_tee_public_key")
    @patch("seismic_web3.client.AsyncHTTPProvider")
    @patch("seismic_web3.client.AsyncWeb3")
    async def test_create_async_client_uses_rpc_url(
        self, mock_async_web3, mock_http, mock_get_tee
    ):
        mock_get_tee.return_value = _MOCK_TEE_PK
        mock_async_web3.return_value = MagicMock()

        cfg = ChainConfig(
            chain_id=1, rpc_url="http://test:8545", ws_url="ws://test:8545"
        )
        await cfg.create_async_client(_TEST_PK)

        mock_http.assert_called_once_with("http://test:8545")

    @patch("seismic_web3.client.async_get_tee_public_key")
    @patch("seismic_web3.client.WebSocketProvider")
    @patch("seismic_web3.client.AsyncWeb3")
    async def test_create_async_client_uses_ws_url(
        self, mock_async_web3, mock_ws, mock_get_tee
    ):
        """When ws=True and ws_url is available, use it."""
        mock_get_tee.return_value = _MOCK_TEE_PK
        mock_async_web3.return_value = MagicMock()

        cfg = ChainConfig(
            chain_id=1, rpc_url="http://test:8545", ws_url="ws://test:8546"
        )
        await cfg.create_async_client(_TEST_PK, ws=True)

        mock_ws.assert_called_once_with("ws://test:8546")

    @patch("seismic_web3.client.async_get_tee_public_key")
    @patch("seismic_web3.client.AsyncHTTPProvider")
    @patch("seismic_web3.client.AsyncWeb3")
    async def test_create_async_client_falls_back_to_rpc(
        self, mock_async_web3, mock_http, mock_get_tee
    ):
        """When ws=True but ws_url is None, fall back to rpc_url."""
        mock_get_tee.return_value = _MOCK_TEE_PK
        mock_async_web3.return_value = MagicMock()

        cfg = ChainConfig(chain_id=1, rpc_url="http://test:8545")
        await cfg.create_async_client(_TEST_PK)

        mock_http.assert_called_once_with("http://test:8545")
