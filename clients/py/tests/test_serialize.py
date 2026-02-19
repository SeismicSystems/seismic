"""Tests for seismic_web3.transaction.serialize — RLP serialization and signing.

Includes a known-vector test against seismic-viem's encoding test suite.
"""

from hexbytes import HexBytes

from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)
from seismic_web3.transaction.serialize import (
    hash_unsigned,
    serialize_signed,
    serialize_unsigned,
    sign_seismic_tx,
)
from seismic_web3.transaction_types import SeismicElements, Signature, UnsignedSeismicTx

# ---------------------------------------------------------------------------
# Test vector from seismic-viem encoding.ts (anvil chainId=31337)
# ---------------------------------------------------------------------------

# The encrypted calldata produced by seismic-viem for this test case.
ENCRYPTED_CALLDATA = HexBytes(
    "0xbf645e68de8096b62950fac2d5bceb71ab1a085aed2e973a8b4f961ca77209f9"
    "9116130edecd27c39fc62e1b3c05ff42d9e4382f987fc55c2011f8e4f2e66204"
    "e17174e9d2756bb20f4cdfe48bd5d237"
)

# Anvil account #0 private key.
ANVIL_PK = PrivateKey(
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)

EXPECTED_SIGNED_TX = (
    "0x4af90112827a6902843b9aca00830186a094d3e8763675e4c425df46cc3b5c0f"
    "6cbdac39604687038d7ea4c68000a1028e76821eb4d77fd30223ca971c49738eb5"
    "b5b71eabe93f96b348fdce788ae5a08c46a2b6020bba77fcb1e676a680a0934207"
    "181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f906480b850"
    "bf645e68de8096b62950fac2d5bceb71ab1a085aed2e973a8b4f961ca77209f991"
    "16130edecd27c39fc62e1b3c05ff42d9e4382f987fc55c2011f8e4f2e66204e171"
    "74e9d2756bb20f4cdfe48bd5d23780a0fea7db32f4e44d75eb13f84d2cf04c2808"
    "a5c8dba8dac629476fe27e04c7629fa001f17d58cf879dc2c787d526b90a17b6d7"
    "bcbf4fbd581215ae3f6099e43c84c5"
)


def _make_test_tx() -> UnsignedSeismicTx:
    """Construct the test transaction matching seismic-viem's encoding test."""
    return UnsignedSeismicTx(
        chain_id=31337,
        nonce=2,
        gas_price=1_000_000_000,
        gas=100_000,
        to="0xd3e8763675e4c425df46cc3b5c0f6cbdac396046",
        value=1_000_000_000_000_000,
        data=ENCRYPTED_CALLDATA,
        seismic=SeismicElements(
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


class TestSerializeUnsigned:
    def test_deterministic(self):
        tx = _make_test_tx()
        assert serialize_unsigned(tx) == serialize_unsigned(tx)

    def test_returns_bytes(self):
        tx = _make_test_tx()
        raw = serialize_unsigned(tx)
        assert isinstance(raw, bytes)
        assert len(raw) > 0


class TestHashUnsigned:
    def test_returns_32_bytes(self):
        tx = _make_test_tx()
        h = hash_unsigned(tx)
        assert len(h) == 32

    def test_deterministic(self):
        tx = _make_test_tx()
        assert hash_unsigned(tx) == hash_unsigned(tx)


class TestSerializeSigned:
    def test_with_known_signature(self):
        """Verify serialize_signed produces the expected bytes with a known sig."""
        tx = _make_test_tx()
        # Extract signature from the known test vector by signing.
        signed = sign_seismic_tx(tx, ANVIL_PK)
        assert signed.to_0x_hex() == EXPECTED_SIGNED_TX

    def test_includes_type_prefix(self):
        tx = _make_test_tx()
        sig = Signature(v=0, r=1, s=2)
        raw = serialize_signed(tx, sig)
        assert raw[0] == 0x4A


class TestSignSeismicTx:
    def test_known_vector(self):
        """Full sign + serialize must match seismic-viem output."""
        tx = _make_test_tx()
        signed = sign_seismic_tx(tx, ANVIL_PK)
        assert signed.to_0x_hex() == EXPECTED_SIGNED_TX

    def test_returns_hexbytes(self):
        tx = _make_test_tx()
        signed = sign_seismic_tx(tx, ANVIL_PK)
        assert isinstance(signed, HexBytes)
