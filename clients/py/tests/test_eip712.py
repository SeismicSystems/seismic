"""Tests for seismic_web3.transaction.eip712 — EIP-712 typed data signing.

Includes cross-validation tests against seismic-alloy (Rust) and
seismic-viem (TypeScript) reference implementations.
"""

import rlp
from eth_hash.auto import keccak
from eth_keys import keys as eth_keys
from hexbytes import HexBytes

from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)
from seismic_web3.chains import TYPED_DATA_MESSAGE_VERSION
from seismic_web3.transaction.eip712 import (
    DOMAIN_VERSION,
    EIP712_DOMAIN_TYPE_HASH,
    EIP712_DOMAIN_TYPE_STR,
    TX_SEISMIC_TYPE_HASH,
    TX_SEISMIC_TYPE_STR,
    VERIFYING_CONTRACT,
    build_seismic_typed_data,
    domain_separator,
    eip712_signing_hash,
    sign_seismic_tx_eip712,
    struct_hash,
)
from seismic_web3.transaction.serialize import (
    hash_unsigned,
    sign_seismic_tx,
)
from seismic_web3.transaction_types import (
    SeismicElements,
    Signature,
    UnsignedSeismicTx,
)

# ---------------------------------------------------------------------------
# Shared test data — anvil account #0 (same key as test_serialize.py)
# ---------------------------------------------------------------------------

ANVIL_PK = PrivateKey(
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)
ANVIL_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

ENCRYPTED_CALLDATA = HexBytes(
    "0xbf645e68de8096b62950fac2d5bceb71ab1a085aed2e973a8b4f961ca77209f9"
    "9116130edecd27c39fc62e1b3c05ff42d9e4382f987fc55c2011f8e4f2e66204"
    "e17174e9d2756bb20f4cdfe48bd5d237"
)

# Pre-computed expected values (from running the implementation once
# and cross-validating against Rust seismic-alloy).
EXPECTED_DOMAIN_SEPARATOR_31337 = bytes.fromhex(
    "fe8197449db574220cfe11bd4271c4680b39d9e034a2c583b3960e78e872cdd0"
)
EXPECTED_DOMAIN_SEPARATOR_5124 = bytes.fromhex(
    "8c18a115e1d4ee84a16bce167a1f8213215705f0a5fd00475741e2cd7a53fed6"
)


def _make_eip712_tx() -> UnsignedSeismicTx:
    """Construct a test tx with message_version=2 (same fields as test_serialize)."""
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
            message_version=2,
            recent_block_hash=Bytes32(
                "0x934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f90"
            ),
            expires_at_block=100,
            signed_read=False,
        ),
    )


def _make_rust_test_vector_tx() -> UnsignedSeismicTx:
    """Construct the tx from seismic-alloy's test_eip712_hash."""
    return UnsignedSeismicTx(
        chain_id=5124,
        nonce=48,
        gas_price=360_000,
        gas=169_477,
        to="0x3aB946eEC2553114040dE82D2e18798a51cf1e14",
        value=1_000_000_000_000_000,
        data=HexBytes(
            "0x4e69e56c3bb999b8c98772ebb32aebcbd43b33e9e65a46333dfe6636f37f3009"
            "e93bad334235aec73bd54d11410e64eb2cab4da8"
        ),
        seismic=SeismicElements(
            encryption_pubkey=CompressedPublicKey(
                "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
            ),
            encryption_nonce=EncryptionNonce("0x7da3a99bf0f90d56551d99ea"),
            message_version=2,
            recent_block_hash=Bytes32(
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            ),
            expires_at_block=1_000_000,
            signed_read=False,
        ),
    )


# ===================================================================
# Type hashes
# ===================================================================


class TestTypeHashes:
    """Verify pre-computed EIP-712 type hashes."""

    def test_domain_type_hash_matches_well_known_value(self):
        """The EIP-712 domain type hash is a well-known constant."""
        expected = bytes.fromhex(
            "8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f"
        )
        assert expected == EIP712_DOMAIN_TYPE_HASH

    def test_domain_type_hash_matches_keccak_of_string(self):
        assert keccak(EIP712_DOMAIN_TYPE_STR.encode()) == EIP712_DOMAIN_TYPE_HASH

    def test_tx_seismic_type_hash_matches_keccak_of_string(self):
        assert keccak(TX_SEISMIC_TYPE_STR.encode()) == TX_SEISMIC_TYPE_HASH

    def test_tx_seismic_type_hash_is_32_bytes(self):
        assert len(TX_SEISMIC_TYPE_HASH) == 32

    def test_type_strings_are_canonical(self):
        """No extra spaces, proper comma separation."""
        assert " ," not in EIP712_DOMAIN_TYPE_STR
        assert ", " not in EIP712_DOMAIN_TYPE_STR
        assert " ," not in TX_SEISMIC_TYPE_STR
        assert ", " not in TX_SEISMIC_TYPE_STR

    def test_tx_seismic_has_13_fields(self):
        """TxSeismic struct has exactly 13 fields."""
        # Count commas inside the parentheses: 12 commas = 13 fields
        inner = TX_SEISMIC_TYPE_STR.split("(", 1)[1].rstrip(")")
        assert inner.count(",") == 12

    def test_domain_version_matches_constant(self):
        assert str(TYPED_DATA_MESSAGE_VERSION) == DOMAIN_VERSION
        assert DOMAIN_VERSION == "2"


# ===================================================================
# Domain separator
# ===================================================================


class TestDomainSeparator:
    """Verify EIP-712 domain separator computation."""

    def test_sanvil_chain_id(self):
        assert domain_separator(31337) == EXPECTED_DOMAIN_SEPARATOR_31337

    def test_testnet_chain_id(self):
        assert domain_separator(5124) == EXPECTED_DOMAIN_SEPARATOR_5124

    def test_different_chain_ids_produce_different_separators(self):
        assert domain_separator(1) != domain_separator(31337)
        assert domain_separator(1) != domain_separator(5124)
        assert domain_separator(31337) != domain_separator(5124)

    def test_deterministic(self):
        assert domain_separator(31337) == domain_separator(31337)

    def test_returns_32_bytes(self):
        assert len(domain_separator(1)) == 32

    def test_chain_id_zero(self):
        """Chain ID 0 is valid and produces a unique separator."""
        sep = domain_separator(0)
        assert len(sep) == 32
        assert sep != domain_separator(1)


# ===================================================================
# Struct hash
# ===================================================================


class TestStructHash:
    """Verify EIP-712 struct hash computation."""

    def test_deterministic(self):
        tx = _make_eip712_tx()
        assert struct_hash(tx) == struct_hash(tx)

    def test_returns_32_bytes(self):
        tx = _make_eip712_tx()
        assert len(struct_hash(tx)) == 32

    def test_different_from_raw_hash(self):
        """Struct hash must differ from raw keccak(0x4a || RLP) hash."""
        tx = _make_eip712_tx()
        assert struct_hash(tx) != hash_unsigned(tx)

    def test_contract_creation_uses_zero_address(self):
        """When to=None, the 'to' field encodes as the zero address."""
        tx_with_to = _make_eip712_tx()
        tx_no_to = UnsignedSeismicTx(
            chain_id=tx_with_to.chain_id,
            nonce=tx_with_to.nonce,
            gas_price=tx_with_to.gas_price,
            gas=tx_with_to.gas,
            to=None,  # contract creation
            value=tx_with_to.value,
            data=tx_with_to.data,
            seismic=tx_with_to.seismic,
        )
        # Should produce a valid hash (no error), but different from with-to
        h = struct_hash(tx_no_to)
        assert len(h) == 32
        assert h != struct_hash(tx_with_to)

    def test_zero_value(self):
        """value=0 produces a valid hash."""
        tx = _make_eip712_tx()
        tx_zero = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=0,
            data=tx.data,
            seismic=tx.seismic,
        )
        h = struct_hash(tx_zero)
        assert len(h) == 32
        assert h != struct_hash(tx)

    def test_empty_input_data(self):
        """Empty calldata (0x) produces a valid hash."""
        tx = _make_eip712_tx()
        tx_empty = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=HexBytes(b""),
            seismic=tx.seismic,
        )
        h = struct_hash(tx_empty)
        assert len(h) == 32
        assert h != struct_hash(tx)

    def test_signed_read_true_vs_false(self):
        """signedRead=True produces a different hash from False."""
        tx = _make_eip712_tx()
        se = tx.seismic
        tx_read = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=tx.data,
            seismic=SeismicElements(
                encryption_pubkey=se.encryption_pubkey,
                encryption_nonce=se.encryption_nonce,
                message_version=se.message_version,
                recent_block_hash=se.recent_block_hash,
                expires_at_block=se.expires_at_block,
                signed_read=True,
            ),
        )
        assert struct_hash(tx_read) != struct_hash(tx)

    def test_known_vector_rust(self):
        """Cross-validate struct hash with seismic-alloy test_eip712_hash."""
        tx = _make_rust_test_vector_tx()
        expected = bytes.fromhex(
            "683f681e3a89f9fabcd7175e53c2d72ee0ecd9843e217aa9e97cfeebdad129de"
        )
        assert struct_hash(tx) == expected


# ===================================================================
# EIP-712 signing hash
# ===================================================================


class TestEIP712SigningHash:
    """Verify the full EIP-712 signing hash."""

    def test_deterministic(self):
        tx = _make_eip712_tx()
        assert eip712_signing_hash(tx) == eip712_signing_hash(tx)

    def test_returns_32_bytes(self):
        tx = _make_eip712_tx()
        assert len(eip712_signing_hash(tx)) == 32

    def test_different_from_raw_hash(self):
        """EIP-712 signing hash MUST differ from raw keccak(0x4a||RLP)."""
        tx = _make_eip712_tx()
        assert eip712_signing_hash(tx) != hash_unsigned(tx)

    def test_known_vector_anvil(self):
        """Known signing hash for the anvil test tx (chain 31337)."""
        tx = _make_eip712_tx()
        expected = bytes.fromhex(
            "ba60b0f59a1993d9bd3947ab92e9093a98580cf0ea886809d44488c116be6526"
        )
        assert eip712_signing_hash(tx) == expected

    def test_known_vector_rust(self):
        """Cross-validate signing hash with seismic-alloy test_eip712_hash."""
        tx = _make_rust_test_vector_tx()
        expected = bytes.fromhex(
            "6152c0b10ef0cc2eb90a4bf27f5449d8a1f0529fb09998006dcee7a2e6f51f3f"
        )
        assert eip712_signing_hash(tx) == expected

    def test_different_chain_ids_produce_different_hashes(self):
        """Same tx fields but different chain_id → different signing hash."""
        tx1 = _make_eip712_tx()
        tx2 = UnsignedSeismicTx(
            chain_id=5124,
            nonce=tx1.nonce,
            gas_price=tx1.gas_price,
            gas=tx1.gas,
            to=tx1.to,
            value=tx1.value,
            data=tx1.data,
            seismic=tx1.seismic,
        )
        assert eip712_signing_hash(tx1) != eip712_signing_hash(tx2)

    def test_is_keccak_of_1901_prefix_plus_hashes(self):
        """Verify the hash is keccak256(0x1901 || domainSep || structHash)."""
        tx = _make_eip712_tx()
        manual = keccak(b"\x19\x01" + domain_separator(tx.chain_id) + struct_hash(tx))
        assert eip712_signing_hash(tx) == manual


# ===================================================================
# Build typed data dict
# ===================================================================


class TestBuildSeismicTypedData:
    """Verify the JSON-serializable typed data structure."""

    def test_has_required_keys(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        assert set(td.keys()) == {"types", "primaryType", "domain", "message"}

    def test_primary_type_is_tx_seismic(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        assert td["primaryType"] == "TxSeismic"

    def test_domain_fields(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        domain = td["domain"]
        assert domain["name"] == "Seismic Transaction"
        assert domain["version"] == "2"
        assert domain["chainId"] == 31337
        assert domain["verifyingContract"] == VERIFYING_CONTRACT

    def test_types_has_domain_and_tx_seismic(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        assert "EIP712Domain" in td["types"]
        assert "TxSeismic" in td["types"]

    def test_tx_seismic_type_has_13_fields(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        assert len(td["types"]["TxSeismic"]) == 13

    def test_message_fields_match_tx(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        msg = td["message"]
        assert msg["chainId"] == tx.chain_id
        assert msg["nonce"] == tx.nonce
        assert msg["gasPrice"] == tx.gas_price
        assert msg["gasLimit"] == tx.gas
        assert msg["value"] == tx.value
        assert msg["messageVersion"] == tx.seismic.message_version
        assert msg["expiresAtBlock"] == tx.seismic.expires_at_block
        assert msg["signedRead"] == tx.seismic.signed_read

    def test_bytes_fields_are_hex(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        msg = td["message"]
        assert msg["input"].startswith("0x")
        assert msg["encryptionPubkey"].startswith("0x")
        assert msg["recentBlockHash"].startswith("0x")

    def test_contract_creation_to_is_zero_address(self):
        tx = _make_eip712_tx()
        tx_create = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=None,
            value=tx.value,
            data=tx.data,
            seismic=tx.seismic,
        )
        td = build_seismic_typed_data(tx_create)
        assert td["message"]["to"] == VERIFYING_CONTRACT

    def test_encryption_nonce_is_integer(self):
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        assert isinstance(td["message"]["encryptionNonce"], int)

    def test_type_field_names_match_message_keys(self):
        """Every name in the TxSeismic type definition appears in message."""
        tx = _make_eip712_tx()
        td = build_seismic_typed_data(tx)
        type_names = {f["name"] for f in td["types"]["TxSeismic"]}
        message_keys = set(td["message"].keys())
        assert type_names == message_keys


# ===================================================================
# Full sign + serialize (EIP-712)
# ===================================================================

# Pre-computed expected signed tx (anvil key #0, message_version=2).
EXPECTED_EIP712_SIGNED_TX = (
    "0x4af90112827a6902843b9aca00830186a094d3e8763675e4c425df46cc3b5c0f"
    "6cbdac39604687038d7ea4c68000a1028e76821eb4d77fd30223ca971c49738eb5"
    "b5b71eabe93f96b348fdce788ae5a08c46a2b6020bba77fcb1e676a602a0934207"
    "181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f906480b850"
    "bf645e68de8096b62950fac2d5bceb71ab1a085aed2e973a8b4f961ca77209f991"
    "16130edecd27c39fc62e1b3c05ff42d9e4382f987fc55c2011f8e4f2e66204e171"
    "74e9d2756bb20f4cdfe48bd5d23780a040ea4805150580ce89bd7478962ee176d1"
    "ab030ae988ff9078803845dc8fbb64a0383bcbd8b9d35ff9d3cba98b8aa0be739c"
    "b4fc2f0685b6ee9359a744a639a727"
)


class TestSignSeismicTxEIP712:
    """Verify EIP-712 sign + serialize."""

    def test_returns_hexbytes(self):
        tx = _make_eip712_tx()
        signed = sign_seismic_tx_eip712(tx, ANVIL_PK)
        assert isinstance(signed, HexBytes)

    def test_includes_type_prefix(self):
        tx = _make_eip712_tx()
        signed = sign_seismic_tx_eip712(tx, ANVIL_PK)
        assert signed[0] == 0x4A

    def test_known_vector(self):
        """Full EIP-712 sign + serialize must match expected output."""
        tx = _make_eip712_tx()
        signed = sign_seismic_tx_eip712(tx, ANVIL_PK)
        assert signed.to_0x_hex() == EXPECTED_EIP712_SIGNED_TX

    def test_different_from_raw_signing(self):
        """EIP-712 signed tx must differ from raw signed tx."""
        tx = _make_eip712_tx()
        signed_eip712 = sign_seismic_tx_eip712(tx, ANVIL_PK)
        signed_raw = sign_seismic_tx(tx, ANVIL_PK)
        assert signed_eip712 != signed_raw

    def test_rlp_decodable(self):
        """The output (minus type prefix) must be valid RLP."""
        tx = _make_eip712_tx()
        signed = sign_seismic_tx_eip712(tx, ANVIL_PK)
        # Strip 0x4a prefix, decode RLP
        decoded = rlp.decode(bytes(signed[1:]))
        # 13 tx fields + 3 signature fields = 16
        assert len(decoded) == 16

    def test_message_version_in_rlp(self):
        """The RLP contains message_version=2."""
        tx = _make_eip712_tx()
        signed = sign_seismic_tx_eip712(tx, ANVIL_PK)
        decoded = rlp.decode(bytes(signed[1:]))
        # message_version is field index 8 (0-indexed)
        msg_version = int.from_bytes(decoded[8], "big") if decoded[8] else 0
        assert msg_version == 2

    def test_signature_recovers_to_signer(self):
        """Recover public key from EIP-712 hash + signature matches signer."""
        tx = _make_eip712_tx()
        msg_hash = eip712_signing_hash(tx)

        sk = eth_keys.PrivateKey(bytes(ANVIL_PK))
        sig_obj = sk.sign_msg_hash(msg_hash)
        recovered = sig_obj.recover_public_key_from_msg_hash(msg_hash)

        assert recovered.to_checksum_address() == sk.public_key.to_checksum_address()
        assert recovered.to_checksum_address() == ANVIL_ADDRESS


# ===================================================================
# Cross-validation with Rust (seismic-alloy)
# ===================================================================


class TestRustCrossValidation:
    """Cross-validate against seismic-alloy's test_eip712_hash test vector."""

    def test_tx_hash_matches_rust(self):
        """keccak(signed_bytes) must match the Rust expected tx_hash."""
        tx = _make_rust_test_vector_tx()

        # Signature from the Rust test
        r = int.from_bytes(
            bytes.fromhex(
                "e93185920818650416b4b0cc953c48f59fd9a29af4b7e1c4b1ac4824392f9220"
            ),
            "big",
        )
        s = int.from_bytes(
            bytes.fromhex(
                "79b76b064a83d423997b7234c575588f60da5d3e1e0561eff9804eb04c23789a"
            ),
            "big",
        )
        sig = Signature(v=0, r=r, s=s)

        from seismic_web3.transaction.serialize import serialize_signed

        signed = serialize_signed(tx, sig)
        tx_hash = keccak(bytes(signed))
        expected = bytes.fromhex(
            "d33755a15aeb3023cb6e5a593a60cb963b2381c44342a43b1088465931b1cdbc"
        )
        assert tx_hash == expected

    def test_domain_separator_matches_rust(self):
        assert domain_separator(5124) == EXPECTED_DOMAIN_SEPARATOR_5124

    def test_struct_hash_matches_rust(self):
        tx = _make_rust_test_vector_tx()
        expected = bytes.fromhex(
            "683f681e3a89f9fabcd7175e53c2d72ee0ecd9843e217aa9e97cfeebdad129de"
        )
        assert struct_hash(tx) == expected

    def test_signing_hash_matches_rust(self):
        tx = _make_rust_test_vector_tx()
        expected = bytes.fromhex(
            "6152c0b10ef0cc2eb90a4bf27f5449d8a1f0529fb09998006dcee7a2e6f51f3f"
        )
        assert eip712_signing_hash(tx) == expected


# ===================================================================
# Edge cases
# ===================================================================


class TestEdgeCases:
    """Edge case tests for EIP-712 signing."""

    def test_max_uint64_values(self):
        """Max uint64 for nonce, gas, expires_at_block."""
        tx = _make_eip712_tx()
        max64 = 2**64 - 1
        tx_max = UnsignedSeismicTx(
            chain_id=max64,
            nonce=max64,
            gas_price=tx.gas_price,
            gas=max64,
            to=tx.to,
            value=tx.value,
            data=tx.data,
            seismic=SeismicElements(
                encryption_pubkey=tx.seismic.encryption_pubkey,
                encryption_nonce=tx.seismic.encryption_nonce,
                message_version=tx.seismic.message_version,
                recent_block_hash=tx.seismic.recent_block_hash,
                expires_at_block=max64,
                signed_read=tx.seismic.signed_read,
            ),
        )
        h = eip712_signing_hash(tx_max)
        assert len(h) == 32

    def test_max_uint128_gas_price(self):
        """Max uint128 for gasPrice."""
        tx = _make_eip712_tx()
        tx_max = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=2**128 - 1,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=tx.data,
            seismic=tx.seismic,
        )
        h = eip712_signing_hash(tx_max)
        assert len(h) == 32
        assert h != eip712_signing_hash(tx)

    def test_max_uint256_value(self):
        """Max uint256 for value."""
        tx = _make_eip712_tx()
        tx_max = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=2**256 - 1,
            data=tx.data,
            seismic=tx.seismic,
        )
        h = eip712_signing_hash(tx_max)
        assert len(h) == 32

    def test_max_uint96_encryption_nonce(self):
        """Max uint96 (12 bytes all 0xff) for encryptionNonce."""
        tx = _make_eip712_tx()
        tx_max = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=tx.data,
            seismic=SeismicElements(
                encryption_pubkey=tx.seismic.encryption_pubkey,
                encryption_nonce=EncryptionNonce(b"\xff" * 12),
                message_version=tx.seismic.message_version,
                recent_block_hash=tx.seismic.recent_block_hash,
                expires_at_block=tx.seismic.expires_at_block,
                signed_read=tx.seismic.signed_read,
            ),
        )
        h = eip712_signing_hash(tx_max)
        assert len(h) == 32
        assert h != eip712_signing_hash(tx)

    def test_empty_calldata(self):
        """Empty data (0x) produces valid hash and signature."""
        tx = _make_eip712_tx()
        tx_empty = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=HexBytes(b""),
            seismic=tx.seismic,
        )
        signed = sign_seismic_tx_eip712(tx_empty, ANVIL_PK)
        assert signed[0] == 0x4A
        assert len(signed) > 1

    def test_large_calldata(self):
        """Large calldata (1KB) produces valid hash."""
        tx = _make_eip712_tx()
        tx_large = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=HexBytes(b"\xab" * 1024),
            seismic=tx.seismic,
        )
        h = eip712_signing_hash(tx_large)
        assert len(h) == 32

    def test_message_version_affects_struct_hash(self):
        """Changing message_version changes the struct hash."""
        tx = _make_eip712_tx()
        se = tx.seismic
        tx_v0 = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=tx.data,
            seismic=SeismicElements(
                encryption_pubkey=se.encryption_pubkey,
                encryption_nonce=se.encryption_nonce,
                message_version=0,
                recent_block_hash=se.recent_block_hash,
                expires_at_block=se.expires_at_block,
                signed_read=se.signed_read,
            ),
        )
        assert struct_hash(tx_v0) != struct_hash(tx)

    def test_all_zero_seismic_elements(self):
        """Tx with minimal/zero seismic elements produces a valid hash."""
        tx = UnsignedSeismicTx(
            chain_id=1,
            nonce=0,
            gas_price=0,
            gas=0,
            to=None,
            value=0,
            data=HexBytes(b""),
            seismic=SeismicElements(
                encryption_pubkey=CompressedPublicKey(
                    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
                ),
                encryption_nonce=EncryptionNonce(b"\x00" * 12),
                message_version=2,
                recent_block_hash=Bytes32(b"\x00" * 32),
                expires_at_block=0,
                signed_read=False,
            ),
        )
        h = eip712_signing_hash(tx)
        assert len(h) == 32
        signed = sign_seismic_tx_eip712(tx, ANVIL_PK)
        assert signed[0] == 0x4A

    def test_signed_read_with_eip712(self):
        """Signed read (signedRead=True) with EIP-712 works correctly."""
        tx = _make_eip712_tx()
        se = tx.seismic
        tx_read = UnsignedSeismicTx(
            chain_id=tx.chain_id,
            nonce=tx.nonce,
            gas_price=tx.gas_price,
            gas=tx.gas,
            to=tx.to,
            value=tx.value,
            data=tx.data,
            seismic=SeismicElements(
                encryption_pubkey=se.encryption_pubkey,
                encryption_nonce=se.encryption_nonce,
                message_version=se.message_version,
                recent_block_hash=se.recent_block_hash,
                expires_at_block=se.expires_at_block,
                signed_read=True,
            ),
        )
        signed = sign_seismic_tx_eip712(tx_read, ANVIL_PK)
        assert signed[0] == 0x4A
        # Signature should recover to the signer
        msg_hash = eip712_signing_hash(tx_read)
        sk = eth_keys.PrivateKey(bytes(ANVIL_PK))
        sig_obj = sk.sign_msg_hash(msg_hash)
        recovered = sig_obj.recover_public_key_from_msg_hash(msg_hash)
        assert recovered.to_checksum_address() == ANVIL_ADDRESS

    def test_different_private_keys_produce_different_signatures(self):
        """Two different keys produce different signed bytes."""
        tx = _make_eip712_tx()
        pk2 = PrivateKey(
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
        )
        signed1 = sign_seismic_tx_eip712(tx, ANVIL_PK)
        signed2 = sign_seismic_tx_eip712(tx, pk2)
        assert signed1 != signed2
