"""Tests for seismic_web3.crypto.aes, crypto.nonce, and transaction.aead."""

import pytest
from cryptography.exceptions import InvalidTag
from hexbytes import HexBytes

from seismic_web3._types import Bytes32, CompressedPublicKey, EncryptionNonce
from seismic_web3.crypto.aes import AesGcmCrypto
from seismic_web3.crypto.nonce import random_encryption_nonce
from seismic_web3.transaction.aead import encode_metadata_as_aad
from seismic_web3.transaction_types import (
    LegacyFields,
    SeismicElements,
    TxSeismicMetadata,
)

# ---------------------------------------------------------------------------
# AES-GCM
# ---------------------------------------------------------------------------

ZERO_KEY = Bytes32(b"\x00" * 32)
ZERO_NONCE = EncryptionNonce(b"\x00" * 12)


class TestAesGcmCrypto:
    def test_known_vector(self):
        """key=zeros(32), nonce=zeros(12), plaintext="HelloAESGCM".

        Expected ciphertext from seismic-viem precompile test.
        """
        crypto = AesGcmCrypto(ZERO_KEY)
        plaintext = HexBytes(b"HelloAESGCM")
        ct = crypto.encrypt(plaintext, ZERO_NONCE)
        assert (
            ct.to_0x_hex() == "0x86c22c5122212e3d400d886f80dfcfcbacb96cbc815db886e1a6cd"
        )

    def test_roundtrip(self):
        key = Bytes32(b"\x01" * 32)
        nonce = EncryptionNonce(b"\x02" * 12)
        crypto = AesGcmCrypto(key)
        plaintext = HexBytes(b"some secret data")
        ct = crypto.encrypt(plaintext, nonce)
        pt = crypto.decrypt(ct, nonce)
        assert pt == plaintext

    def test_roundtrip_with_aad(self):
        key = Bytes32(b"\xaa" * 32)
        nonce = EncryptionNonce(b"\xbb" * 12)
        aad = b"extra context"
        crypto = AesGcmCrypto(key)
        plaintext = HexBytes(b"authenticated data")
        ct = crypto.encrypt(plaintext, nonce, aad=aad)
        pt = crypto.decrypt(ct, nonce, aad=aad)
        assert pt == plaintext

    def test_wrong_aad_fails(self):
        key = Bytes32(b"\xcc" * 32)
        nonce = EncryptionNonce(b"\xdd" * 12)
        crypto = AesGcmCrypto(key)
        ct = crypto.encrypt(HexBytes(b"data"), nonce, aad=b"correct")
        with pytest.raises(InvalidTag):
            crypto.decrypt(ct, nonce, aad=b"wrong")

    def test_empty_plaintext(self):
        crypto = AesGcmCrypto(ZERO_KEY)
        ct = crypto.encrypt(HexBytes(b""), ZERO_NONCE)
        assert ct == HexBytes(b"")

    def test_empty_ciphertext(self):
        crypto = AesGcmCrypto(ZERO_KEY)
        pt = crypto.decrypt(HexBytes(b""), ZERO_NONCE)
        assert pt == HexBytes(b"")


# ---------------------------------------------------------------------------
# Nonce generation
# ---------------------------------------------------------------------------


class TestRandomEncryptionNonce:
    def test_length(self):
        nonce = random_encryption_nonce()
        assert len(nonce) == 12

    def test_no_leading_zero(self):
        for _ in range(50):
            nonce = random_encryption_nonce()
            assert nonce[0] != 0

    def test_type(self):
        nonce = random_encryption_nonce()
        assert isinstance(nonce, EncryptionNonce)

    def test_unique(self):
        nonces = {random_encryption_nonce() for _ in range(20)}
        assert len(nonces) == 20


# ---------------------------------------------------------------------------
# AAD encoding
# ---------------------------------------------------------------------------


MOCK_SENDER = "0xd3e8763675e4C425df46CC3B5c0f6cbDAC396046"
MOCK_PUBKEY = CompressedPublicKey("0x02" + "ab" * 32)
MOCK_ENC_NONCE = EncryptionNonce("0x" + "ff" * 12)
MOCK_BLOCK_HASH = Bytes32("0x" + "cc" * 32)


class TestEncodeMetadataAsAAD:
    def _make_metadata(self) -> TxSeismicMetadata:
        return TxSeismicMetadata(
            sender=MOCK_SENDER,
            legacy_fields=LegacyFields(
                chain_id=31337,
                nonce=2,
                to=MOCK_SENDER,
                value=1000,
            ),
            seismic_elements=SeismicElements(
                encryption_pubkey=MOCK_PUBKEY,
                encryption_nonce=MOCK_ENC_NONCE,
                message_version=0,
                recent_block_hash=MOCK_BLOCK_HASH,
                expires_at_block=100,
                signed_read=False,
            ),
        )

    def test_returns_bytes(self):
        aad = encode_metadata_as_aad(self._make_metadata())
        assert isinstance(aad, bytes)
        assert len(aad) > 0

    def test_deterministic(self):
        meta = self._make_metadata()
        assert encode_metadata_as_aad(meta) == encode_metadata_as_aad(meta)

    def test_different_metadata_different_aad(self):
        meta1 = self._make_metadata()
        meta2 = TxSeismicMetadata(
            sender=MOCK_SENDER,
            legacy_fields=LegacyFields(
                chain_id=31337,
                nonce=3,  # different nonce
                to=MOCK_SENDER,
                value=1000,
            ),
            seismic_elements=meta1.seismic_elements,
        )
        assert encode_metadata_as_aad(meta1) != encode_metadata_as_aad(meta2)
