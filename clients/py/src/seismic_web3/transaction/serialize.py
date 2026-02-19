"""Seismic transaction serialization and signing.

Provides RLP serialization of ``TxSeismic`` (type ``0x4a``) transactions,
matching the Rust ``alloy-rlp`` encoding and the TypeScript
``serializeSeismicTransaction`` in seismic-viem.

Key design choice: we bypass ``eth-account``'s ``TypedTransaction``
dispatcher entirely, using ``eth_keys`` for raw ECDSA signing and
manual RLP serialization.  This avoids forking ``eth-account`` to
register a custom transaction type.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import rlp
from eth_hash.auto import keccak
from eth_keys import keys as eth_keys
from hexbytes import HexBytes

from seismic_web3.chains import SEISMIC_TX_TYPE
from seismic_web3.transaction_types import Signature

if TYPE_CHECKING:
    from seismic_web3._types import PrivateKey
    from seismic_web3.transaction_types import UnsignedSeismicTx


def _int_to_rlp_bytes(value: int) -> bytes:
    """Encode an integer as minimal big-endian bytes for RLP.

    Zero is encoded as empty bytes (``b""``), matching Rust/viem
    convention.
    """
    if value == 0:
        return b""
    return value.to_bytes((value.bit_length() + 7) // 8, "big")


def _address_to_bytes(address: str | None) -> bytes:
    """Convert a checksummed address to 20 raw bytes.  ``None`` -> ``b""``."""
    if address is None:
        return b""
    return bytes.fromhex(address[2:])


def _bool_to_rlp_bytes(value: bool) -> bytes:
    """Encode boolean: ``True`` -> ``b"\\x01"``, ``False`` -> ``b""``."""
    return b"\x01" if value else b""


def _tx_rlp_fields(tx: UnsignedSeismicTx) -> list[bytes]:
    """Build the ordered list of RLP fields for a Seismic transaction.

    Field order matches ``serializeSeismicTransaction`` in seismic-viem::

        chainId, nonce, gasPrice, gas, to, value,
        encPubkey, encNonce, msgVersion, recentBlockHash,
        expiresAtBlock, signedRead, data
    """
    se = tx.seismic
    return [
        _int_to_rlp_bytes(tx.chain_id),
        _int_to_rlp_bytes(tx.nonce),
        _int_to_rlp_bytes(tx.gas_price),
        _int_to_rlp_bytes(tx.gas),
        _address_to_bytes(tx.to),
        _int_to_rlp_bytes(tx.value),
        bytes(se.encryption_pubkey),
        bytes(se.encryption_nonce),
        _int_to_rlp_bytes(se.message_version),
        bytes(se.recent_block_hash),
        _int_to_rlp_bytes(se.expires_at_block),
        _bool_to_rlp_bytes(se.signed_read),
        bytes(tx.data),
    ]


def serialize_unsigned(tx: UnsignedSeismicTx) -> bytes:
    """RLP-encode an unsigned ``TxSeismic`` (no type prefix).

    Args:
        tx: The unsigned Seismic transaction.

    Returns:
        RLP-encoded byte string (without the ``0x4a`` prefix).
    """
    return rlp.encode(_tx_rlp_fields(tx))


def serialize_signed(tx: UnsignedSeismicTx, sig: Signature) -> HexBytes:
    """Serialize a signed ``TxSeismic``: ``0x4a`` prefix + RLP(fields + v,r,s).

    The signature is appended as three additional RLP items:
    ``yParity``, ``r``, ``s`` (with leading-zero trimming).

    Args:
        tx: The unsigned Seismic transaction.
        sig: ECDSA signature components.

    Returns:
        Full signed transaction bytes (ready for ``eth_sendRawTransaction``).
    """
    fields = _tx_rlp_fields(tx)
    fields.append(_int_to_rlp_bytes(sig.v))
    fields.append(_int_to_rlp_bytes(sig.r))
    fields.append(_int_to_rlp_bytes(sig.s))

    encoded = rlp.encode(fields)
    return HexBytes(bytes([SEISMIC_TX_TYPE]) + encoded)


def hash_unsigned(tx: UnsignedSeismicTx) -> bytes:
    """Keccak-256 of ``0x4a`` + ``serialize_unsigned(tx)``.

    This is the message hash that gets signed.

    Args:
        tx: The unsigned Seismic transaction.

    Returns:
        32-byte Keccak-256 digest.
    """
    payload = bytes([SEISMIC_TX_TYPE]) + serialize_unsigned(tx)
    return keccak(payload)


def sign_seismic_tx(tx: UnsignedSeismicTx, private_key: PrivateKey) -> HexBytes:
    """Sign and serialize a Seismic transaction.

    Steps:
        1. Compute ``hash_unsigned(tx)``
        2. Sign with ``eth_keys.PrivateKey.sign_msg_hash()``
        3. Serialize with ``serialize_signed(tx, sig)``

    Uses ``eth_keys`` (already a web3.py dependency) for ECDSA signing,
    bypassing ``eth-account``'s ``TypedTransaction`` dispatcher.

    Args:
        tx: The unsigned Seismic transaction.
        private_key: 32-byte secp256k1 private key.

    Returns:
        Full signed transaction bytes.
    """
    msg_hash = hash_unsigned(tx)
    sk = eth_keys.PrivateKey(bytes(private_key))
    sig_obj = sk.sign_msg_hash(msg_hash)

    sig = Signature(
        v=sig_obj.v,
        r=sig_obj.r,
        s=sig_obj.s,
    )
    return serialize_signed(tx, sig)
