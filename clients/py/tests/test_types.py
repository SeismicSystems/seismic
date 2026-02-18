"""Tests for seismic_web3._types — primitive byte types."""

import pytest
from hexbytes import HexBytes

from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)

# ---------------------------------------------------------------------------
# Bytes32
# ---------------------------------------------------------------------------


class TestBytes32:
    VALID_HEX = "0x" + "ab" * 32
    VALID_BYTES = b"\xab" * 32

    def test_from_hex_string(self):
        b = Bytes32(self.VALID_HEX)
        assert len(b) == 32
        assert isinstance(b, bytes)

    def test_from_raw_bytes(self):
        b = Bytes32(self.VALID_BYTES)
        assert len(b) == 32

    def test_from_hexbytes(self):
        hb = HexBytes(self.VALID_HEX)
        b = Bytes32(hb)
        assert len(b) == 32

    def test_to_0x_hex(self):
        b = Bytes32(self.VALID_HEX)
        assert b.to_0x_hex() == self.VALID_HEX

    def test_isinstance_bytes(self):
        b = Bytes32(self.VALID_HEX)
        assert isinstance(b, bytes)
        assert isinstance(b, HexBytes)

    def test_too_short_raises(self):
        with pytest.raises(ValueError, match="expected 32 bytes, got 16"):
            Bytes32("0x" + "ab" * 16)

    def test_too_long_raises(self):
        with pytest.raises(ValueError, match="expected 32 bytes, got 33"):
            Bytes32("0x" + "ab" * 33)

    def test_immutable(self):
        b = Bytes32(self.VALID_HEX)
        with pytest.raises(TypeError):
            b[0] = 0  # type: ignore[index]

    def test_hashable(self):
        b = Bytes32(self.VALID_HEX)
        assert hash(b) == hash(self.VALID_BYTES)
        assert b in {b}


# ---------------------------------------------------------------------------
# PrivateKey
# ---------------------------------------------------------------------------


class TestPrivateKey:
    def test_is_bytes32(self):
        pk = PrivateKey("0x" + "01" * 32)
        assert isinstance(pk, Bytes32)
        assert len(pk) == 32

    def test_wrong_length_raises(self):
        with pytest.raises(ValueError, match="expected 32 bytes"):
            PrivateKey("0x" + "01" * 31)


# ---------------------------------------------------------------------------
# CompressedPublicKey
# ---------------------------------------------------------------------------


class TestCompressedPublicKey:
    VALID_02 = "0x02" + "ab" * 32
    VALID_03 = "0x03" + "cd" * 32

    def test_prefix_02(self):
        cpk = CompressedPublicKey(self.VALID_02)
        assert len(cpk) == 33
        assert cpk[0] == 0x02

    def test_prefix_03(self):
        cpk = CompressedPublicKey(self.VALID_03)
        assert len(cpk) == 33
        assert cpk[0] == 0x03

    def test_bad_prefix_raises(self):
        bad = "0x04" + "ab" * 32
        with pytest.raises(ValueError, match="must start with 0x02 or 0x03"):
            CompressedPublicKey(bad)

    def test_wrong_length_raises(self):
        with pytest.raises(ValueError, match="expected 33 bytes"):
            CompressedPublicKey("0x02" + "ab" * 31)

    def test_isinstance_bytes(self):
        cpk = CompressedPublicKey(self.VALID_02)
        assert isinstance(cpk, bytes)


# ---------------------------------------------------------------------------
# EncryptionNonce
# ---------------------------------------------------------------------------


class TestEncryptionNonce:
    VALID_HEX = "0x" + "ff" * 12
    VALID_BYTES = b"\xff" * 12

    def test_from_hex(self):
        n = EncryptionNonce(self.VALID_HEX)
        assert len(n) == 12

    def test_from_bytes(self):
        n = EncryptionNonce(self.VALID_BYTES)
        assert len(n) == 12

    def test_wrong_length_raises(self):
        with pytest.raises(ValueError, match="expected 12 bytes"):
            EncryptionNonce("0x" + "ff" * 11)

    def test_to_0x_hex(self):
        n = EncryptionNonce(self.VALID_HEX)
        assert n.to_0x_hex() == self.VALID_HEX
