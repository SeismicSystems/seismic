"""EIP-712 typed data signing for Seismic transactions.

Implements `EIP-712 <https://eips.ethereum.org/EIPS/eip-712>`_ structured
data hashing and signing for ``TxSeismic`` (type ``0x4a``) transactions
with ``message_version=2``.

This is an alternative to the raw signing mode (``message_version=0``)
in :mod:`seismic_web3.transaction.serialize`.  The RLP serialization of
the signed transaction is identical in both modes — only the hash that
gets signed differs.  The Seismic node checks ``message_version`` and
uses the matching verification path.

Reference implementations:
    * TypeScript: ``seismic-viem/src/signSeismicTypedData.ts``
    * Rust: ``seismic-alloy/crates/consensus/src/transaction/seismic.rs``
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from eth_hash.auto import keccak
from eth_keys import keys as eth_keys
from hexbytes import HexBytes

from seismic_web3.chains import TYPED_DATA_MESSAGE_VERSION
from seismic_web3.transaction.serialize import serialize_signed
from seismic_web3.transaction_types import Signature

if TYPE_CHECKING:
    from seismic_web3._types import PrivateKey
    from seismic_web3.transaction_types import UnsignedSeismicTx

# ---------------------------------------------------------------------------
# EIP-712 type strings (canonical — no extra spaces)
# ---------------------------------------------------------------------------

#: Canonical EIP-712 domain type string.
EIP712_DOMAIN_TYPE_STR: str = (
    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
)

#: Canonical EIP-712 type string for ``TxSeismic``.
TX_SEISMIC_TYPE_STR: str = (
    "TxSeismic("
    "uint64 chainId,"
    "uint64 nonce,"
    "uint128 gasPrice,"
    "uint64 gasLimit,"
    "address to,"
    "uint256 value,"
    "bytes input,"
    "bytes encryptionPubkey,"
    "uint96 encryptionNonce,"
    "uint8 messageVersion,"
    "bytes32 recentBlockHash,"
    "uint64 expiresAtBlock,"
    "bool signedRead"
    ")"
)

# ---------------------------------------------------------------------------
# Domain constants
# ---------------------------------------------------------------------------

#: EIP-712 domain name for Seismic transactions.
DOMAIN_NAME: str = "Seismic Transaction"

#: EIP-712 domain version (matches ``TYPED_DATA_MESSAGE_VERSION``).
DOMAIN_VERSION: str = str(TYPED_DATA_MESSAGE_VERSION)

#: Verifying contract address (zero — signing happens off-chain via RPC).
VERIFYING_CONTRACT: str = "0x0000000000000000000000000000000000000000"

# ---------------------------------------------------------------------------
# Pre-computed type hashes
# ---------------------------------------------------------------------------

#: ``keccak256(EIP712_DOMAIN_TYPE_STR)`` — the well-known domain type hash.
EIP712_DOMAIN_TYPE_HASH: bytes = keccak(EIP712_DOMAIN_TYPE_STR.encode())

#: ``keccak256(TX_SEISMIC_TYPE_STR)`` — type hash for the TxSeismic struct.
TX_SEISMIC_TYPE_HASH: bytes = keccak(TX_SEISMIC_TYPE_STR.encode())

# Pre-computed hashes of domain string values (they never change).
_DOMAIN_NAME_HASH: bytes = keccak(DOMAIN_NAME.encode())
_DOMAIN_VERSION_HASH: bytes = keccak(DOMAIN_VERSION.encode())


# ---------------------------------------------------------------------------
# Encoding helpers
# ---------------------------------------------------------------------------


def _pad32_int(value: int) -> bytes:
    """Encode an integer as 32-byte big-endian (EIP-712 ``encodeData``)."""
    return value.to_bytes(32, "big")


def _pad32_address(address: str | None) -> bytes:
    """Encode an address as 32-byte left-padded.  ``None`` → zero address."""
    if address is None:
        return b"\x00" * 32
    raw = bytes.fromhex(address[2:] if address.startswith("0x") else address)
    return b"\x00" * 12 + raw


def _pad32_bool(value: bool) -> bytes:
    """Encode a boolean as 32-byte big-endian (0 or 1)."""
    return _pad32_int(1 if value else 0)


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def domain_separator(chain_id: int) -> bytes:
    """Compute the EIP-712 domain separator for a given chain ID.

    .. code-block:: text

        keccak256(
            typeHash(EIP712Domain)
            ‖ keccak256("Seismic Transaction")
            ‖ keccak256("2")
            ‖ abi.encode(uint256, chainId)
            ‖ abi.encode(address, 0x0…0)
        )

    Args:
        chain_id: Numeric chain identifier.

    Returns:
        32-byte keccak256 hash.
    """
    return keccak(
        EIP712_DOMAIN_TYPE_HASH
        + _DOMAIN_NAME_HASH
        + _DOMAIN_VERSION_HASH
        + _pad32_int(chain_id)
        + _pad32_address(VERIFYING_CONTRACT)
    )


def struct_hash(tx: UnsignedSeismicTx) -> bytes:
    """Compute the EIP-712 struct hash for a ``TxSeismic``.

    Dynamic types (``bytes``) are hashed with ``keccak256``.
    Static types are left-padded to 32 bytes.

    Args:
        tx: The unsigned Seismic transaction.

    Returns:
        32-byte keccak256 hash.
    """
    se = tx.seismic
    enc_nonce_int = int.from_bytes(bytes(se.encryption_nonce), "big")

    return keccak(
        TX_SEISMIC_TYPE_HASH
        + _pad32_int(tx.chain_id)  # uint64
        + _pad32_int(tx.nonce)  # uint64
        + _pad32_int(tx.gas_price)  # uint128
        + _pad32_int(tx.gas)  # uint64 (gasLimit)
        + _pad32_address(tx.to)  # address
        + _pad32_int(tx.value)  # uint256
        + keccak(bytes(tx.data))  # bytes (dynamic)
        + keccak(bytes(se.encryption_pubkey))  # bytes (dynamic)
        + _pad32_int(enc_nonce_int)  # uint96
        + _pad32_int(se.message_version)  # uint8
        + bytes(se.recent_block_hash)  # bytes32 (already 32 bytes)
        + _pad32_int(se.expires_at_block)  # uint64
        + _pad32_bool(se.signed_read)  # bool
    )


def eip712_signing_hash(tx: UnsignedSeismicTx) -> bytes:
    r"""Compute the EIP-712 signing hash for a Seismic transaction.

    .. code-block:: text

        keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)

    This is the 32-byte message hash that gets ECDSA-signed.

    Args:
        tx: The unsigned Seismic transaction.

    Returns:
        32-byte keccak256 digest.
    """
    return keccak(b"\x19\x01" + domain_separator(tx.chain_id) + struct_hash(tx))


def build_seismic_typed_data(tx: UnsignedSeismicTx) -> dict[str, Any]:
    """Build the EIP-712 typed data dict for a Seismic transaction.

    Returns a JSON-serializable dictionary matching the format expected
    by ``eth_signTypedData_v4`` (MetaMask / WalletConnect).  Useful for
    display or integration with external signers.

    Args:
        tx: The unsigned Seismic transaction.

    Returns:
        Dict with keys ``types``, ``primaryType``, ``domain``, ``message``.
    """
    se = tx.seismic
    enc_nonce_int = int.from_bytes(bytes(se.encryption_nonce), "big")

    return {
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"},
            ],
            "TxSeismic": [
                {"name": "chainId", "type": "uint64"},
                {"name": "nonce", "type": "uint64"},
                {"name": "gasPrice", "type": "uint128"},
                {"name": "gasLimit", "type": "uint64"},
                {"name": "to", "type": "address"},
                {"name": "value", "type": "uint256"},
                {"name": "input", "type": "bytes"},
                {"name": "encryptionPubkey", "type": "bytes"},
                {"name": "encryptionNonce", "type": "uint96"},
                {"name": "messageVersion", "type": "uint8"},
                {"name": "recentBlockHash", "type": "bytes32"},
                {"name": "expiresAtBlock", "type": "uint64"},
                {"name": "signedRead", "type": "bool"},
            ],
        },
        "primaryType": "TxSeismic",
        "domain": {
            "name": DOMAIN_NAME,
            "version": DOMAIN_VERSION,
            "chainId": tx.chain_id,
            "verifyingContract": VERIFYING_CONTRACT,
        },
        "message": {
            "chainId": tx.chain_id,
            "nonce": tx.nonce,
            "gasPrice": tx.gas_price,
            "gasLimit": tx.gas,
            "to": tx.to or VERIFYING_CONTRACT,
            "value": tx.value,
            "input": HexBytes(tx.data).to_0x_hex(),
            "encryptionPubkey": HexBytes(bytes(se.encryption_pubkey)).to_0x_hex(),
            "encryptionNonce": enc_nonce_int,
            "messageVersion": se.message_version,
            "recentBlockHash": HexBytes(bytes(se.recent_block_hash)).to_0x_hex(),
            "expiresAtBlock": se.expires_at_block,
            "signedRead": se.signed_read,
        },
    }


def sign_seismic_tx_eip712(tx: UnsignedSeismicTx, private_key: PrivateKey) -> HexBytes:
    """Sign and serialize a Seismic transaction using EIP-712 typed data.

    Steps:
        1. Compute :func:`eip712_signing_hash` (instead of ``hash_unsigned``)
        2. Sign with ``eth_keys.PrivateKey.sign_msg_hash()``
        3. Serialize with ``serialize_signed(tx, sig)`` — same RLP as raw

    The RLP serialization is identical to raw signing; only the ECDSA
    message hash differs.  The Seismic node checks ``message_version``
    to decide which verification path to use.

    Args:
        tx: The unsigned Seismic transaction (should have
            ``message_version == 2``).
        private_key: 32-byte secp256k1 private key.

    Returns:
        Full signed transaction bytes (``0x4a`` prefix + RLP).
    """
    msg_hash = eip712_signing_hash(tx)
    sk = eth_keys.PrivateKey(bytes(private_key))
    sig_obj = sk.sign_msg_hash(msg_hash)

    sig = Signature(
        v=sig_obj.v,
        r=sig_obj.r,
        s=sig_obj.s,
    )
    return serialize_signed(tx, sig)
