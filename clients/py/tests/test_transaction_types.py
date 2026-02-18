"""Tests for seismic_web3.transaction_types — Seismic tx data structures."""

import pytest
from hexbytes import HexBytes

from seismic_web3._types import Bytes32, CompressedPublicKey, EncryptionNonce
from seismic_web3.transaction_types import (
    LegacyFields,
    SeismicElements,
    SeismicSecurityParams,
    Signature,
    TxSeismicMetadata,
    UnsignedSeismicTx,
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

MOCK_PUBKEY = CompressedPublicKey("0x02" + "ab" * 32)
MOCK_NONCE = EncryptionNonce("0x" + "ff" * 12)
MOCK_BLOCK_HASH = Bytes32("0x" + "cc" * 32)
MOCK_SENDER = "0xd3e8763675e4C425df46CC3B5c0f6cbDAC396046"


def _make_seismic_elements() -> SeismicElements:
    return SeismicElements(
        encryption_pubkey=MOCK_PUBKEY,
        encryption_nonce=MOCK_NONCE,
        message_version=0,
        recent_block_hash=MOCK_BLOCK_HASH,
        expires_at_block=1000,
        signed_read=False,
    )


# ---------------------------------------------------------------------------
# Signature
# ---------------------------------------------------------------------------


class TestSignature:
    def test_creation(self):
        sig = Signature(v=27, r=123, s=456)
        assert sig.v == 27
        assert sig.r == 123
        assert sig.s == 456

    def test_frozen(self):
        sig = Signature(v=0, r=1, s=2)
        with pytest.raises(AttributeError):
            sig.v = 1  # type: ignore[misc]


# ---------------------------------------------------------------------------
# SeismicElements
# ---------------------------------------------------------------------------


class TestSeismicElements:
    def test_creation(self):
        se = _make_seismic_elements()
        assert se.encryption_pubkey == MOCK_PUBKEY
        assert se.encryption_nonce == MOCK_NONCE
        assert se.message_version == 0
        assert se.expires_at_block == 1000
        assert se.signed_read is False

    def test_frozen(self):
        se = _make_seismic_elements()
        with pytest.raises(AttributeError):
            se.signed_read = True  # type: ignore[misc]


# ---------------------------------------------------------------------------
# LegacyFields
# ---------------------------------------------------------------------------


class TestLegacyFields:
    def test_creation(self):
        lf = LegacyFields(chain_id=5124, nonce=42, to=MOCK_SENDER, value=0)
        assert lf.chain_id == 5124
        assert lf.nonce == 42
        assert lf.to == MOCK_SENDER
        assert lf.value == 0

    def test_to_none_for_deploy(self):
        lf = LegacyFields(chain_id=5124, nonce=0, to=None, value=0)
        assert lf.to is None


# ---------------------------------------------------------------------------
# TxSeismicMetadata
# ---------------------------------------------------------------------------


class TestTxSeismicMetadata:
    def test_creation(self):
        meta = TxSeismicMetadata(
            sender=MOCK_SENDER,
            legacy_fields=LegacyFields(chain_id=5124, nonce=1, to=MOCK_SENDER, value=0),
            seismic_elements=_make_seismic_elements(),
        )
        assert meta.sender == MOCK_SENDER
        assert meta.legacy_fields.chain_id == 5124
        assert meta.seismic_elements.message_version == 0


# ---------------------------------------------------------------------------
# UnsignedSeismicTx
# ---------------------------------------------------------------------------


class TestUnsignedSeismicTx:
    def test_creation(self):
        tx = UnsignedSeismicTx(
            chain_id=31_337,
            nonce=2,
            gas_price=1_000_000_000,
            gas=100_000,
            to=MOCK_SENDER,
            value=1_000_000_000_000_000,
            data=HexBytes("0xdeadbeef"),
            seismic=_make_seismic_elements(),
        )
        assert tx.chain_id == 31_337
        assert tx.gas == 100_000
        assert tx.data == HexBytes("0xdeadbeef")
        assert isinstance(tx.seismic, SeismicElements)

    def test_frozen(self):
        tx = UnsignedSeismicTx(
            chain_id=1,
            nonce=0,
            gas_price=0,
            gas=0,
            to=None,
            value=0,
            data=HexBytes(b""),
            seismic=_make_seismic_elements(),
        )
        with pytest.raises(AttributeError):
            tx.nonce = 5  # type: ignore[misc]


# ---------------------------------------------------------------------------
# SeismicSecurityParams
# ---------------------------------------------------------------------------


class TestSeismicSecurityParams:
    def test_all_defaults_none(self):
        sp = SeismicSecurityParams()
        assert sp.blocks_window is None
        assert sp.encryption_nonce is None
        assert sp.recent_block_hash is None
        assert sp.expires_at_block is None

    def test_custom_values(self):
        sp = SeismicSecurityParams(
            blocks_window=200,
            encryption_nonce=MOCK_NONCE,
            expires_at_block=5000,
        )
        assert sp.blocks_window == 200
        assert sp.encryption_nonce == MOCK_NONCE
        assert sp.expires_at_block == 5000
