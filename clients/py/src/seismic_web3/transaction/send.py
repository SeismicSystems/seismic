"""Shielded transaction sending and signed reads.

Provides the full pipeline for sending encrypted ``TxSeismic``
transactions and executing signed reads (``eth_call`` with
encrypted calldata).  Both sync and async variants are provided.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

from eth_keys import keys as eth_keys
from hexbytes import HexBytes
from web3.types import RPCEndpoint

from seismic_web3.crypto.nonce import random_encryption_nonce
from seismic_web3.transaction.metadata import (
    DEFAULT_BLOCKS_WINDOW,
    MetadataParams,
    async_build_metadata,
    build_metadata,
)
from seismic_web3.transaction.serialize import sign_seismic_tx
from seismic_web3.transaction_types import (
    DebugWriteResult,
    PlaintextTx,
    UnsignedSeismicTx,
)

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from web3 import AsyncWeb3, Web3
    from web3.types import RPCResponse

    from seismic_web3._types import PrivateKey
    from seismic_web3.client import EncryptionState
    from seismic_web3.transaction_types import SeismicSecurityParams, TxSeismicMetadata

#: Default gas limit when not specified.
_DEFAULT_GAS = 30_000_000


def _address_from_key(private_key: PrivateKey) -> ChecksumAddress:
    """Derive the checksummed Ethereum address from a private key.

    Args:
        private_key: 32-byte secp256k1 private key.

    Returns:
        Checksummed address string (``"0x…"``).
    """
    sk = eth_keys.PrivateKey(bytes(private_key))
    return cast("ChecksumAddress", sk.public_key.to_checksum_address())


def _build_metadata_params(
    private_key: PrivateKey,
    encryption: EncryptionState,
    to: ChecksumAddress | None,
    value: int,
    security: SeismicSecurityParams | None,
    signed_read: bool = False,
) -> MetadataParams:
    """Build ``MetadataParams`` from user-facing arguments.

    Resolves encryption nonce (random if not provided) and
    security overrides.

    Args:
        private_key: Signing key (used to derive sender address).
        encryption: Encryption state.
        to: Recipient address (``None`` for contract creation).
        value: Wei to transfer.
        security: Optional security parameter overrides.
        signed_read: ``True`` for signed ``eth_call`` reads.

    Returns:
        Populated ``MetadataParams``.
    """
    sender = _address_from_key(private_key)
    enc_nonce = (
        security.encryption_nonce if security else None
    ) or random_encryption_nonce()
    blocks_window = (
        security.blocks_window if security else None
    ) or DEFAULT_BLOCKS_WINDOW

    return MetadataParams(
        sender=sender,
        to=to,
        encryption_pubkey=encryption.encryption_pubkey,
        value=value,
        encryption_nonce=enc_nonce,
        blocks_window=blocks_window,
        recent_block_hash=security.recent_block_hash if security else None,
        expires_at_block=security.expires_at_block if security else None,
        signed_read=signed_read,
    )


# ---------------------------------------------------------------------------
# Raw send helpers
# ---------------------------------------------------------------------------


def _check_rpc_response(response: RPCResponse) -> str:
    """Extract result from an RPC response, raising on errors."""
    if "error" in response:
        error = response["error"]
        raise RuntimeError(f"RPC error: {error['message']}")
    return str(response["result"])


def send_shielded_raw(w3: Web3, signed_tx: HexBytes) -> HexBytes:
    """Submit a signed Seismic tx via ``eth_sendRawTransaction`` (sync).

    Args:
        w3: Sync ``Web3`` instance.
        signed_tx: Signed transaction bytes.

    Returns:
        Transaction hash.
    """
    response = w3.provider.make_request(
        RPCEndpoint("eth_sendRawTransaction"), [signed_tx.to_0x_hex()]
    )
    return HexBytes(_check_rpc_response(response))


async def async_send_shielded_raw(w3: AsyncWeb3, signed_tx: HexBytes) -> HexBytes:
    """Submit a signed Seismic tx via ``eth_sendRawTransaction`` (async).

    Args:
        w3: Async ``AsyncWeb3`` instance.
        signed_tx: Signed transaction bytes.

    Returns:
        Transaction hash.
    """
    response = await w3.provider.make_request(
        RPCEndpoint("eth_sendRawTransaction"), [signed_tx.to_0x_hex()]
    )
    return HexBytes(_check_rpc_response(response))


# ---------------------------------------------------------------------------
# Shielded transaction preparation (build + encrypt + sign)
# ---------------------------------------------------------------------------


def _prepare_shielded_transaction(
    w3: Web3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
) -> tuple[HexBytes, UnsignedSeismicTx, TxSeismicMetadata]:
    """Build, encrypt, and sign a shielded transaction (sync).

    Returns the signed bytes, the unsigned tx, and metadata -- but
    does **not** broadcast.

    Returns:
        ``(signed_tx_bytes, unsigned_tx, metadata)``
    """
    params = _build_metadata_params(private_key, encryption, to, value, security)
    metadata = build_metadata(w3, params)

    encrypted = encryption.encrypt(
        data, metadata.seismic_elements.encryption_nonce, metadata
    )

    resolved_gas_price = gas_price if gas_price is not None else w3.eth.gas_price
    resolved_gas = gas if gas is not None else _DEFAULT_GAS

    tx = UnsignedSeismicTx(
        chain_id=metadata.legacy_fields.chain_id,
        nonce=metadata.legacy_fields.nonce,
        gas_price=resolved_gas_price,
        gas=resolved_gas,
        to=to,
        value=value,
        data=HexBytes(encrypted),
        seismic=metadata.seismic_elements,
    )

    signed = sign_seismic_tx(tx, private_key)
    return signed, tx, metadata


async def _async_prepare_shielded_transaction(
    w3: AsyncWeb3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
) -> tuple[HexBytes, UnsignedSeismicTx, TxSeismicMetadata]:
    """Build, encrypt, and sign a shielded transaction (async).

    Returns:
        ``(signed_tx_bytes, unsigned_tx, metadata)``
    """
    params = _build_metadata_params(private_key, encryption, to, value, security)
    metadata = await async_build_metadata(w3, params)

    encrypted = encryption.encrypt(
        data, metadata.seismic_elements.encryption_nonce, metadata
    )

    resolved_gas_price = gas_price if gas_price is not None else await w3.eth.gas_price
    resolved_gas = gas if gas is not None else _DEFAULT_GAS

    tx = UnsignedSeismicTx(
        chain_id=metadata.legacy_fields.chain_id,
        nonce=metadata.legacy_fields.nonce,
        gas_price=resolved_gas_price,
        gas=resolved_gas,
        to=to,
        value=value,
        data=HexBytes(encrypted),
        seismic=metadata.seismic_elements,
    )

    signed = sign_seismic_tx(tx, private_key)
    return signed, tx, metadata


# ---------------------------------------------------------------------------
# Shielded transaction send (full pipeline)
# ---------------------------------------------------------------------------


def send_shielded_transaction(
    w3: Web3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
) -> HexBytes:
    """Send a shielded transaction (sync).

    Full pipeline: build metadata -> encrypt calldata -> build tx ->
    sign -> send raw transaction.

    Args:
        w3: Sync ``Web3`` instance.
        encryption: Encryption state (from :func:`get_encryption`).
        private_key: 32-byte signing key.
        to: Recipient address.
        data: Plaintext calldata (will be encrypted).
        value: Wei to transfer (default ``0``).
        gas: Gas limit.  Uses ``30_000_000`` if not specified.
        gas_price: Gas price in wei.  Fetched from chain if not specified.
        security: Optional security parameter overrides.

    Returns:
        Transaction hash.
    """
    signed, _, _ = _prepare_shielded_transaction(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=to,
        data=data,
        value=value,
        gas=gas,
        gas_price=gas_price,
        security=security,
    )
    return send_shielded_raw(w3, signed)


async def async_send_shielded_transaction(
    w3: AsyncWeb3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
) -> HexBytes:
    """Send a shielded transaction (async).

    Same pipeline as :func:`send_shielded_transaction` but with
    async chain state fetching.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        to: Recipient address.
        data: Plaintext calldata (will be encrypted).
        value: Wei to transfer (default ``0``).
        gas: Gas limit.  Uses ``30_000_000`` if not specified.
        gas_price: Gas price in wei.  Fetched from chain if not specified.
        security: Optional security parameter overrides.

    Returns:
        Transaction hash.
    """
    signed, _, _ = await _async_prepare_shielded_transaction(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=to,
        data=data,
        value=value,
        gas=gas,
        gas_price=gas_price,
        security=security,
    )
    return await async_send_shielded_raw(w3, signed)


# ---------------------------------------------------------------------------
# Debug shielded transaction (send + return plaintext/shielded views)
# ---------------------------------------------------------------------------


def debug_send_shielded_transaction(
    w3: Web3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
) -> DebugWriteResult:
    """Send a shielded transaction and return debug info (sync).

    Same as :func:`send_shielded_transaction` but also returns
    the plaintext and encrypted transaction views for debugging.

    Args:
        w3: Sync ``Web3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        to: Recipient address.
        data: Plaintext calldata (will be encrypted).
        value: Wei to transfer (default ``0``).
        gas: Gas limit.  Uses ``30_000_000`` if not specified.
        gas_price: Gas price in wei.  Fetched from chain if not specified.
        security: Optional security parameter overrides.

    Returns:
        :class:`~seismic_web3.transaction_types.DebugWriteResult`
        with plaintext tx, shielded tx, and transaction hash.
    """
    signed, unsigned_tx, _metadata = _prepare_shielded_transaction(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=to,
        data=data,
        value=value,
        gas=gas,
        gas_price=gas_price,
        security=security,
    )
    tx_hash = send_shielded_raw(w3, signed)

    plaintext_tx = PlaintextTx(
        to=to,
        data=data,
        nonce=unsigned_tx.nonce,
        gas=unsigned_tx.gas,
        gas_price=unsigned_tx.gas_price,
        value=value,
    )
    return DebugWriteResult(
        plaintext_tx=plaintext_tx,
        shielded_tx=unsigned_tx,
        tx_hash=tx_hash,
    )


async def async_debug_send_shielded_transaction(
    w3: AsyncWeb3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
) -> DebugWriteResult:
    """Send a shielded transaction and return debug info (async).

    Same as :func:`async_send_shielded_transaction` but also returns
    the plaintext and encrypted transaction views for debugging.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        to: Recipient address.
        data: Plaintext calldata (will be encrypted).
        value: Wei to transfer (default ``0``).
        gas: Gas limit.  Uses ``30_000_000`` if not specified.
        gas_price: Gas price in wei.  Fetched from chain if not specified.
        security: Optional security parameter overrides.

    Returns:
        :class:`~seismic_web3.transaction_types.DebugWriteResult`
        with plaintext tx, shielded tx, and transaction hash.
    """
    signed, unsigned_tx, _metadata = await _async_prepare_shielded_transaction(
        w3,
        encryption=encryption,
        private_key=private_key,
        to=to,
        data=data,
        value=value,
        gas=gas,
        gas_price=gas_price,
        security=security,
    )
    tx_hash = await async_send_shielded_raw(w3, signed)

    plaintext_tx = PlaintextTx(
        to=to,
        data=data,
        nonce=unsigned_tx.nonce,
        gas=unsigned_tx.gas,
        gas_price=unsigned_tx.gas_price,
        value=value,
    )
    return DebugWriteResult(
        plaintext_tx=plaintext_tx,
        shielded_tx=unsigned_tx,
        tx_hash=tx_hash,
    )


# ---------------------------------------------------------------------------
# Signed reads (eth_call with encrypted calldata)
# ---------------------------------------------------------------------------


def signed_call(
    w3: Web3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int = _DEFAULT_GAS,
    security: SeismicSecurityParams | None = None,
) -> HexBytes | None:
    """Execute a signed read (sync).

    Encrypts calldata, signs the transaction, sends it as an
    ``eth_call``, and decrypts the response.

    Args:
        w3: Sync ``Web3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        to: Contract address to call.
        data: Plaintext calldata (will be encrypted).
        value: Wei to include (default ``0``).
        gas: Gas limit (default ``30_000_000``).
        security: Optional security parameter overrides.

    Returns:
        Decrypted response bytes, or ``None`` if the response is empty.
    """
    params = _build_metadata_params(
        private_key, encryption, to, value, security, signed_read=True
    )
    metadata = build_metadata(w3, params)

    encrypted = encryption.encrypt(
        data, metadata.seismic_elements.encryption_nonce, metadata
    )

    gas_price = w3.eth.gas_price

    tx = UnsignedSeismicTx(
        chain_id=metadata.legacy_fields.chain_id,
        nonce=metadata.legacy_fields.nonce,
        gas_price=gas_price,
        gas=gas,
        to=to,
        value=value,
        data=HexBytes(encrypted),
        seismic=metadata.seismic_elements,
    )

    signed = sign_seismic_tx(tx, private_key)

    # Send signed raw tx directly as first param to eth_call
    # (matches viem: params = [serializedTransaction, block])
    response = w3.provider.make_request(
        RPCEndpoint("eth_call"),
        [signed.to_0x_hex(), "latest"],
    )
    raw_result: str = response.get("result", "0x")

    if not raw_result or raw_result == "0x":
        return None

    result_bytes = HexBytes(raw_result)
    return encryption.decrypt(
        result_bytes,
        metadata.seismic_elements.encryption_nonce,
        metadata,
    )


async def async_signed_call(
    w3: AsyncWeb3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int = _DEFAULT_GAS,
    security: SeismicSecurityParams | None = None,
) -> HexBytes | None:
    """Execute a signed read (async).

    Same pipeline as :func:`signed_call` but with async chain
    state fetching and RPC calls.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        to: Contract address to call.
        data: Plaintext calldata (will be encrypted).
        value: Wei to include (default ``0``).
        gas: Gas limit (default ``30_000_000``).
        security: Optional security parameter overrides.

    Returns:
        Decrypted response bytes, or ``None`` if the response is empty.
    """
    params = _build_metadata_params(
        private_key, encryption, to, value, security, signed_read=True
    )
    metadata = await async_build_metadata(w3, params)

    encrypted = encryption.encrypt(
        data, metadata.seismic_elements.encryption_nonce, metadata
    )

    gas_price = await w3.eth.gas_price

    tx = UnsignedSeismicTx(
        chain_id=metadata.legacy_fields.chain_id,
        nonce=metadata.legacy_fields.nonce,
        gas_price=gas_price,
        gas=gas,
        to=to,
        value=value,
        data=HexBytes(encrypted),
        seismic=metadata.seismic_elements,
    )

    signed = sign_seismic_tx(tx, private_key)

    # Send signed raw tx directly as first param to eth_call
    # (matches viem: params = [serializedTransaction, block])
    response = await w3.provider.make_request(
        RPCEndpoint("eth_call"),
        [signed.to_0x_hex(), "latest"],
    )
    raw_result: str = response.get("result", "0x")

    if not raw_result or raw_result == "0x":
        return None

    result_bytes = HexBytes(raw_result)
    return encryption.decrypt(
        result_bytes,
        metadata.seismic_elements.encryption_nonce,
        metadata,
    )
