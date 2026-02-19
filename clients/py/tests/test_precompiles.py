"""Tests for precompile param encoding, result decoding, gas, and validation.

These test the pure-computation parts of each precompile without
needing a running Seismic node.
"""

import pytest

from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)
from seismic_web3.precompiles._base import (
    calc_linear_gas_cost,
    calc_linear_gas_cost_u32,
    calldata_gas_cost,
)
from seismic_web3.precompiles.aes import (
    AesGcmDecryptParams,
    AesGcmEncryptParams,
    _decrypt_encode,
    _decrypt_gas,
    _encrypt_decode,
    _encrypt_encode,
    _encrypt_gas,
    _nonce_to_bytes,
)
from seismic_web3.precompiles.ecdh import (
    EcdhParams,
    _ecdh_encode,
    _ecdh_gas_cost,
)
from seismic_web3.precompiles.hkdf import _hkdf_decode, _hkdf_encode, _hkdf_gas_cost
from seismic_web3.precompiles.rng import (
    RngParams,
    _rng_decode,
    _rng_encode,
    _rng_gas_cost,
)
from seismic_web3.precompiles.secp256k1 import (
    Secp256k1SigParams,
    _hash_message,
    _sig_encode,
    _sig_gas_cost,
)

# ---------------------------------------------------------------------------
# Gas helpers
# ---------------------------------------------------------------------------


class TestCalldataGasCost:
    def test_all_zeros(self):
        data = b"\x00" * 10
        # 4 * 10 + 12 * 0 = 40
        assert calldata_gas_cost(data) == 40

    def test_all_nonzero(self):
        data = b"\xff" * 10
        # 4 * 10 + 12 * 10 = 160
        assert calldata_gas_cost(data) == 160

    def test_mixed(self):
        data = b"\x00\xff\x00\xff"
        # 4 * 4 + 12 * 2 = 40
        assert calldata_gas_cost(data) == 40

    def test_empty(self):
        assert calldata_gas_cost(b"") == 0


class TestCalcLinearGasCost:
    def test_empty_input(self):
        assert calc_linear_gas_cost(bus=32, length=0, base=100, word=10) == 100

    def test_exact_boundary(self):
        # 32 bytes = 1 word
        assert calc_linear_gas_cost(bus=32, length=32, base=100, word=10) == 110

    def test_partial_word(self):
        # 33 bytes = ceil(33/32) = 2 words
        assert calc_linear_gas_cost(bus=32, length=33, base=100, word=10) == 120

    def test_u32_shortcut(self):
        result = calc_linear_gas_cost_u32(length=64, base=100, word=10)
        expected = calc_linear_gas_cost(bus=32, length=64, base=100, word=10)
        assert result == expected


# ---------------------------------------------------------------------------
# RNG
# ---------------------------------------------------------------------------


class TestRngEncoding:
    def test_encode_num_bytes_only(self):
        data = _rng_encode(RngParams(num_bytes=16))
        assert data == (16).to_bytes(4, "big")
        assert len(data) == 4

    def test_encode_with_pers(self):
        pers = b"seed"
        data = _rng_encode(RngParams(num_bytes=32, pers=pers))
        assert data[:4] == (32).to_bytes(4, "big")
        assert data[4:] == pers
        assert len(data) == 8

    def test_rejects_zero(self):
        with pytest.raises(ValueError, match="num_bytes must be 1-32"):
            _rng_encode(RngParams(num_bytes=0))

    def test_rejects_over_32(self):
        with pytest.raises(ValueError, match="num_bytes must be 1-32"):
            _rng_encode(RngParams(num_bytes=33))

    def test_decode_pads_and_returns_int(self):
        # 32 bytes of 0xff = 2^256 - 1
        result = b"\xff" * 32
        assert _rng_decode(result) == (2**256) - 1

    def test_decode_short_result(self):
        # 1 byte -> should be padded
        assert _rng_decode(b"\x0a") == 10


class TestRngGasCost:
    def test_minimal_no_pers(self):
        params = RngParams(num_bytes=1)
        # init = 3500 + ceil(0/32)*5 = 3500
        # fill = 0 + ceil(1/32)*5 = 5
        assert _rng_gas_cost(params) == 3505

    def test_32_bytes_no_pers(self):
        params = RngParams(num_bytes=32)
        # init = 3500
        # fill = ceil(32/32)*5 = 5
        assert _rng_gas_cost(params) == 3505

    def test_with_pers(self):
        params = RngParams(num_bytes=16, pers=b"x" * 64)
        # init = 3500 + ceil(64/32)*5 = 3510
        # fill = ceil(16/32)*5 = 5
        assert _rng_gas_cost(params) == 3515


# ---------------------------------------------------------------------------
# ECDH
# ---------------------------------------------------------------------------

# Test vectors reused from test_crypto.py
_TEST_SK = PrivateKey(
    "0xa30363336e1bb949185292a2a302de86e447d98f3a43d823c8c234d9e3e5ad77"
)
_TEST_PK = CompressedPublicKey(
    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
)


class TestEcdhEncoding:
    def test_encode_length(self):
        data = _ecdh_encode(EcdhParams(sk=_TEST_SK, pk=_TEST_PK))
        # 32 (sk) + 33 (pk) = 65
        assert len(data) == 65

    def test_encode_layout(self):
        data = _ecdh_encode(EcdhParams(sk=_TEST_SK, pk=_TEST_PK))
        assert data[:32] == bytes(_TEST_SK)
        assert data[32:] == bytes(_TEST_PK)


class TestEcdhGasCost:
    def test_constant(self):
        params = EcdhParams(sk=_TEST_SK, pk=_TEST_PK)
        assert _ecdh_gas_cost(params) == 3120


# ---------------------------------------------------------------------------
# AES
# ---------------------------------------------------------------------------


class TestAesEncoding:
    def test_encrypt_layout(self):
        key = Bytes32(b"\x00" * 32)
        nonce = 1
        plaintext = b"hello"
        data = _encrypt_encode(AesGcmEncryptParams(key, nonce, plaintext))
        assert len(data) == 32 + 12 + 5
        assert data[:32] == b"\x00" * 32
        assert data[32:44] == (1).to_bytes(12, "big")
        assert data[44:] == b"hello"

    def test_decrypt_layout(self):
        key = Bytes32(b"\x01" * 32)
        nonce = EncryptionNonce(b"\x02" * 12)
        ct = b"\xab" * 32
        data = _decrypt_encode(AesGcmDecryptParams(key, nonce, ct))
        assert len(data) == 32 + 12 + 32
        assert data[:32] == b"\x01" * 32
        assert data[32:44] == b"\x02" * 12
        assert data[44:] == ct

    def test_nonce_int_to_bytes(self):
        assert _nonce_to_bytes(256) == (256).to_bytes(12, "big")
        assert len(_nonce_to_bytes(0)) == 12

    def test_nonce_encryption_nonce_passthrough(self):
        nonce = EncryptionNonce(b"\x03" * 12)
        assert _nonce_to_bytes(nonce) == b"\x03" * 12

    def test_encrypt_decode(self):
        raw = b"\xaa\xbb\xcc"
        result = _encrypt_decode(raw)
        assert bytes(result) == raw


class TestAesGasCost:
    def test_empty_plaintext(self):
        params = AesGcmEncryptParams(Bytes32(b"\x00" * 32), 0, b"")
        # 1000 + 0 blocks = 1000
        assert _encrypt_gas(params) == 1000

    def test_16_bytes(self):
        params = AesGcmEncryptParams(Bytes32(b"\x00" * 32), 0, b"\x00" * 16)
        # 1000 + ceil(16/16)*30 = 1030
        assert _encrypt_gas(params) == 1030

    def test_17_bytes(self):
        params = AesGcmEncryptParams(Bytes32(b"\x00" * 32), 0, b"\x00" * 17)
        # 1000 + ceil(17/16)*30 = 1060
        assert _encrypt_gas(params) == 1060

    def test_decrypt_gas_matches_encrypt(self):
        ct = b"\x00" * 32
        d_params = AesGcmDecryptParams(Bytes32(b"\x00" * 32), 0, ct)
        e_params = AesGcmEncryptParams(Bytes32(b"\x00" * 32), 0, ct)
        assert _decrypt_gas(d_params) == _encrypt_gas(e_params)


# ---------------------------------------------------------------------------
# HKDF
# ---------------------------------------------------------------------------


class TestHkdfEncoding:
    def test_passthrough(self):
        ikm = b"input key material"
        assert _hkdf_encode(ikm) == ikm

    def test_decode_returns_bytes32(self):
        raw = b"\xaa" * 32 + b"\x00" * 10  # extra bytes ignored
        result = _hkdf_decode(raw)
        assert isinstance(result, Bytes32)
        assert len(result) == 32
        assert bytes(result) == b"\xaa" * 32


class TestHkdfGasCost:
    def test_empty_input(self):
        # linear = 3000 + 0*12 = 3000
        # total = 2*3000 + 120 = 6120
        assert _hkdf_gas_cost(b"") == 6120

    def test_32_bytes(self):
        # linear = 3000 + 1*12 = 3012
        # total = 2*3012 + 120 = 6144
        assert _hkdf_gas_cost(b"\x00" * 32) == 6144


# ---------------------------------------------------------------------------
# secp256k1
# ---------------------------------------------------------------------------


class TestSecp256k1Encoding:
    def test_encode_length(self):
        sk = PrivateKey(b"\x01" * 32)
        msg_hash = Bytes32(b"\x02" * 32)
        data = _sig_encode(Secp256k1SigParams(sk, msg_hash))
        # ABI-encoded: 2 x bytes32 = 64 bytes
        assert len(data) == 64

    def test_encode_values(self):
        sk = PrivateKey(b"\x01" * 32)
        msg_hash = Bytes32(b"\x02" * 32)
        data = _sig_encode(Secp256k1SigParams(sk, msg_hash))
        assert data[:32] == b"\x01" * 32
        assert data[32:] == b"\x02" * 32


class TestSecp256k1GasCost:
    def test_constant(self):
        sk = PrivateKey(b"\x01" * 32)
        msg_hash = Bytes32(b"\x02" * 32)
        assert _sig_gas_cost(Secp256k1SigParams(sk, msg_hash)) == 3000


class TestHashMessage:
    def test_known_hash(self):
        """Verify EIP-191 hash matches the known Ethereum personal_sign hash."""
        from eth_hash.auto import keccak

        msg = "hello"
        expected_prefix = b"\x19Ethereum Signed Message:\n5"
        expected = keccak(expected_prefix + b"hello")
        assert bytes(_hash_message(msg)) == expected

    def test_returns_bytes32(self):
        result = _hash_message("test")
        assert isinstance(result, Bytes32)
        assert len(result) == 32
