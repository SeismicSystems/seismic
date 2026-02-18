"""Tests for seismic_web3.client — EncryptionState and get_encryption."""

from hexbytes import HexBytes

from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)
from seismic_web3.client import EncryptionState, get_encryption
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
        from seismic_web3.crypto.secp import private_key_to_compressed_public_key

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
