"""Tests for seismic_web3.crypto — ECDH, key derivation, and secp utils.

Test vectors are taken from seismic-viem's test suite to ensure
cross-implementation compatibility.
"""

from seismic_web3._types import Bytes32, CompressedPublicKey, PrivateKey
from seismic_web3.crypto.ecdh import (
    derive_aes_key,
    generate_aes_key,
    shared_key_from_point,
    shared_secret_point,
)
from seismic_web3.crypto.secp import (
    compress_public_key,
    private_key_to_compressed_public_key,
)

# ---------------------------------------------------------------------------
# Test vectors from seismic-viem (aesKeygen.ts, encoding.ts)
# ---------------------------------------------------------------------------

# Network (TEE) compressed public key.
TEE_PK_HEX = "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
TEE_PK = CompressedPublicKey(TEE_PK_HEX)

# Encryption private key.
ENC_SK_HEX = "0xa30363336e1bb949185292a2a302de86e447d98f3a43d823c8c234d9e3e5ad77"
ENC_SK = PrivateKey(ENC_SK_HEX)

# Expected shared secret point (64 bytes, no 04 prefix).
EXPECTED_SHARED_POINT_HEX = (
    "0xae50584c10ef7484c2f28868cce536958960ab86376f1bd7d6c44fcf52e1a18c"
    "347bab95860cbd4a9fec067be4217ce48d83964e3d85bdf6b9384a30a44f0653"
)

# Expected shared key (32 bytes) after Rust-compatible extraction.
EXPECTED_SHARED_KEY_HEX = (
    "0x46a4d6fce8eca748ba8362e726de51a5c62202c887d6bb81fa6f4df1fb360999"
)

# Expected AES-256 key after HKDF.
EXPECTED_AES_KEY_HEX = (
    "0xbf0dd6556618d1bf8d1602bf80be3a0f7cc729973829bb9acb75bd77770d5b90"
)

# Private key whose compressed public key matches TEE_PK.
KEYGEN_SK_HEX = "0x311d54d3bf8359c70827122a44a7b4458733adce3c51c6b59d9acfce85e07505"
KEYGEN_SK = PrivateKey(KEYGEN_SK_HEX)


# ---------------------------------------------------------------------------
# ECDH pipeline
# ---------------------------------------------------------------------------


class TestSharedSecretPoint:
    def test_known_vector(self):
        point = shared_secret_point(ENC_SK, TEE_PK)
        assert len(point) == 64
        expected = bytes.fromhex(EXPECTED_SHARED_POINT_HEX[2:])
        assert point == expected


class TestSharedKeyFromPoint:
    def test_known_vector(self):
        point = bytes.fromhex(EXPECTED_SHARED_POINT_HEX[2:])
        key = shared_key_from_point(point)
        assert isinstance(key, Bytes32)
        assert key.to_0x_hex() == EXPECTED_SHARED_KEY_HEX


class TestDeriveAesKey:
    def test_known_vector(self):
        shared_key = Bytes32(EXPECTED_SHARED_KEY_HEX)
        aes_key = derive_aes_key(shared_key)
        assert isinstance(aes_key, Bytes32)
        assert aes_key.to_0x_hex() == EXPECTED_AES_KEY_HEX


class TestGenerateAesKey:
    def test_full_pipeline(self):
        aes_key = generate_aes_key(ENC_SK, TEE_PK)
        assert isinstance(aes_key, Bytes32)
        assert aes_key.to_0x_hex() == EXPECTED_AES_KEY_HEX


# ---------------------------------------------------------------------------
# secp256k1 key utilities
# ---------------------------------------------------------------------------


class TestPrivateKeyToCompressedPublicKey:
    def test_known_vector(self):
        cpk = private_key_to_compressed_public_key(KEYGEN_SK)
        assert isinstance(cpk, CompressedPublicKey)
        assert len(cpk) == 33
        assert cpk.to_0x_hex() == TEE_PK_HEX


class TestCompressPublicKey:
    def test_roundtrip(self):
        """Compress a key derived from a known private key."""
        from coincurve import PublicKey as _CoincurvePublicKey

        pt = _CoincurvePublicKey.from_secret(bytes(KEYGEN_SK))
        uncompressed = pt.format(compressed=False)  # 65 bytes
        compressed = compress_public_key(uncompressed)
        assert compressed.to_0x_hex() == TEE_PK_HEX

    def test_wrong_length_raises(self):
        import pytest

        with pytest.raises(ValueError, match="65-byte"):
            compress_public_key(b"\x04" + b"\x00" * 32)
