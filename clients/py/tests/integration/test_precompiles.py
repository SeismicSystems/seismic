"""Integration tests for Mercury EVM precompiles.

These tests call the actual on-chain precompiles against a running
sanvil or seismic-reth node.
"""

import pytest
from web3 import AsyncWeb3, Web3

from seismic_web3._types import Bytes32, PrivateKey
from seismic_web3.crypto.secp import private_key_to_compressed_public_key
from seismic_web3.precompiles import (
    aes_gcm_decrypt,
    aes_gcm_encrypt,
    async_aes_gcm_decrypt,
    async_aes_gcm_encrypt,
    async_ecdh,
    async_hkdf,
    async_rng,
    async_secp256k1_sign,
    ecdh,
    hkdf,
    rng,
    secp256k1_sign,
)

# Re-use the dev key from conftest
_DEV_SK = PrivateKey(
    bytes.fromhex("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
)


# ---------------------------------------------------------------------------
# RNG
# ---------------------------------------------------------------------------


class TestRng:
    def test_returns_nonzero(self, w3: Web3):
        result = rng(w3, num_bytes=32)
        assert result > 0

    def test_respects_num_bytes(self, w3: Web3):
        result = rng(w3, num_bytes=1)
        assert 0 <= result <= 0xFF

    def test_with_personalization(self, w3: Web3):
        result = rng(w3, num_bytes=16, pers=b"test-seed")
        assert result >= 0

    @pytest.mark.asyncio
    async def test_async(self, async_w3: AsyncWeb3):
        result = await async_rng(async_w3, num_bytes=32)
        assert result > 0


# ---------------------------------------------------------------------------
# ECDH
# ---------------------------------------------------------------------------


class TestEcdh:
    def test_returns_shared_secret(self, w3: Web3):
        sk = PrivateKey(b"\x01" * 32)
        other_sk = PrivateKey(b"\x02" * 32)
        pk = private_key_to_compressed_public_key(other_sk)
        result = ecdh(w3, sk=sk, pk=pk)
        assert isinstance(result, Bytes32)
        assert len(result) == 32

    def test_matches_client_side(self, w3: Web3):
        """On-chain ECDH should produce the same result as client-side."""
        from seismic_web3.crypto.ecdh import generate_aes_key

        sk = PrivateKey(b"\x01" * 32)
        other_sk = PrivateKey(b"\x02" * 32)
        pk = private_key_to_compressed_public_key(other_sk)

        on_chain = ecdh(w3, sk=sk, pk=pk)
        client_side = generate_aes_key(sk, pk)

        # The on-chain ECDH precompile does ECDH + HKDF together,
        # which matches the client-side generate_aes_key pipeline.
        assert bytes(on_chain) == bytes(client_side)

    @pytest.mark.asyncio
    async def test_async(self, async_w3: AsyncWeb3):
        sk = PrivateKey(b"\x01" * 32)
        other_sk = PrivateKey(b"\x02" * 32)
        pk = private_key_to_compressed_public_key(other_sk)
        result = await async_ecdh(async_w3, sk=sk, pk=pk)
        assert isinstance(result, Bytes32)


# ---------------------------------------------------------------------------
# AES-GCM
# ---------------------------------------------------------------------------


class TestAesGcm:
    def test_encrypt_decrypt_roundtrip(self, w3: Web3):
        key = Bytes32(b"\x00" * 32)
        nonce = 1
        plaintext = b"Hello, precompile!"
        ct = aes_gcm_encrypt(w3, aes_key=key, nonce=nonce, plaintext=plaintext)
        pt = aes_gcm_decrypt(w3, aes_key=key, nonce=nonce, ciphertext=bytes(ct))
        assert bytes(pt) == plaintext

    def test_different_keys_produce_different_ciphertext(self, w3: Web3):
        nonce = 42
        plaintext = b"secret"
        ct1 = aes_gcm_encrypt(
            w3, aes_key=Bytes32(b"\x00" * 32), nonce=nonce, plaintext=plaintext
        )
        ct2 = aes_gcm_encrypt(
            w3, aes_key=Bytes32(b"\x01" * 32), nonce=nonce, plaintext=plaintext
        )
        assert ct1 != ct2

    @pytest.mark.asyncio
    async def test_async_roundtrip(self, async_w3: AsyncWeb3):
        key = Bytes32(b"\x00" * 32)
        nonce = 1
        plaintext = b"async test"
        ct = await async_aes_gcm_encrypt(
            async_w3, aes_key=key, nonce=nonce, plaintext=plaintext
        )
        pt = await async_aes_gcm_decrypt(
            async_w3, aes_key=key, nonce=nonce, ciphertext=bytes(ct)
        )
        assert bytes(pt) == plaintext


# ---------------------------------------------------------------------------
# HKDF
# ---------------------------------------------------------------------------


class TestHkdf:
    def test_deterministic(self, w3: Web3):
        ikm = b"input key material"
        r1 = hkdf(w3, ikm)
        r2 = hkdf(w3, ikm)
        assert r1 == r2
        assert isinstance(r1, Bytes32)
        assert len(r1) == 32

    def test_different_inputs_different_outputs(self, w3: Web3):
        r1 = hkdf(w3, b"input-1")
        r2 = hkdf(w3, b"input-2")
        assert r1 != r2

    @pytest.mark.asyncio
    async def test_async(self, async_w3: AsyncWeb3):
        result = await async_hkdf(async_w3, b"async key material")
        assert isinstance(result, Bytes32)


# ---------------------------------------------------------------------------
# secp256k1 Sign
# ---------------------------------------------------------------------------


class TestSecp256k1Sign:
    def test_returns_signature(self, w3: Web3):
        sig = secp256k1_sign(w3, sk=_DEV_SK, message="hello world")
        # Expect at least r(32) + s(32) = 64 bytes
        assert len(sig) >= 64

    @pytest.mark.asyncio
    async def test_async(self, async_w3: AsyncWeb3):
        sig = await async_secp256k1_sign(async_w3, sk=_DEV_SK, message="hello world")
        assert len(sig) >= 64
