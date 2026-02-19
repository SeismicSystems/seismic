"""SRC20 event watching with viewing keys.

Ports ``seismic-viem/src/actions/src20/watchSRC20Events.ts`` and
``watchSRC20EventsWithKey.ts``.

Provides polling-based event watchers that fetch logs via
``eth_getLogs``, decrypt encrypted amounts using an AES-256 viewing
key, and invoke user callbacks for Transfer and Approval events.
"""

from __future__ import annotations

import asyncio
import contextlib
import logging
import threading
import time
from typing import TYPE_CHECKING, Any

from eth_abi import decode as abi_decode
from eth_hash.auto import keccak
from hexbytes import HexBytes
from web3 import Web3

from seismic_web3._types import Bytes32
from seismic_web3.src20.crypto import decrypt_encrypted_amount
from seismic_web3.src20.directory import compute_key_hash, get_viewing_key
from seismic_web3.src20.types import (
    DecryptedApprovalLog,
    DecryptedTransferLog,
)

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from web3 import AsyncWeb3

    from seismic_web3._types import PrivateKey
    from seismic_web3.client import EncryptionState
    from seismic_web3.src20.types import (
        ApprovalCallback,
        AsyncApprovalCallback,
        AsyncErrorCallback,
        AsyncTransferCallback,
        ErrorCallback,
        TransferCallback,
    )

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Event topic hashes
# ---------------------------------------------------------------------------

#: ``keccak256("Transfer(address,address,bytes32,bytes)")``
TRANSFER_TOPIC: bytes = keccak(b"Transfer(address,address,bytes32,bytes)")

#: ``keccak256("Approval(address,address,bytes32,bytes)")``
APPROVAL_TOPIC: bytes = keccak(b"Approval(address,address,bytes32,bytes)")


# ---------------------------------------------------------------------------
# Log decoding helpers
# ---------------------------------------------------------------------------


def _address_from_topic(topic: bytes) -> ChecksumAddress:
    """Extract a checksummed address from a 32-byte log topic."""
    return Web3.to_checksum_address(topic[-20:])


def _decode_log(
    log: dict[str, Any],
    aes_key: Bytes32,
) -> DecryptedTransferLog | DecryptedApprovalLog | None:
    """Decode and decrypt a single raw log entry.

    Returns ``None`` if the event topic doesn't match Transfer/Approval.
    """
    topics: list[HexBytes] = log["topics"]
    if len(topics) < 4:
        return None

    event_sig = bytes(topics[0])
    data_bytes: bytes = bytes(HexBytes(log["data"]))

    # ABI-decode the non-indexed ``encryptedAmount`` (dynamic ``bytes``)
    (encrypted_amount,) = abi_decode(["bytes"], data_bytes)

    decrypted_amount = decrypt_encrypted_amount(aes_key, encrypted_amount)

    block_number = (
        log["blockNumber"]
        if isinstance(log["blockNumber"], int)
        else int(log["blockNumber"], 16)
    )
    tx_hash = HexBytes(log["transactionHash"])
    encrypt_key_hash = bytes(topics[3])

    if event_sig == TRANSFER_TOPIC:
        return DecryptedTransferLog(
            from_address=_address_from_topic(bytes(topics[1])),
            to_address=_address_from_topic(bytes(topics[2])),
            encrypt_key_hash=encrypt_key_hash,
            encrypted_amount=encrypted_amount,
            decrypted_amount=decrypted_amount,
            transaction_hash=tx_hash,
            block_number=block_number,
        )

    if event_sig == APPROVAL_TOPIC:
        return DecryptedApprovalLog(
            owner=_address_from_topic(bytes(topics[1])),
            spender=_address_from_topic(bytes(topics[2])),
            encrypt_key_hash=encrypt_key_hash,
            encrypted_amount=encrypted_amount,
            decrypted_amount=decrypted_amount,
            transaction_hash=tx_hash,
            block_number=block_number,
        )

    return None


def _build_filter_params(
    token_address: ChecksumAddress | None,
    encrypt_key_hash: bytes,
    from_block: int,
    to_block: int | str,
) -> dict[str, Any]:
    """Build ``eth_getLogs`` filter parameters."""
    params: dict[str, Any] = {
        "fromBlock": hex(from_block) if isinstance(from_block, int) else from_block,
        "toBlock": hex(to_block) if isinstance(to_block, int) else to_block,
        "topics": [
            [HexBytes(TRANSFER_TOPIC).hex(), HexBytes(APPROVAL_TOPIC).hex()],
            None,  # any from/owner
            None,  # any to/spender
            HexBytes(encrypt_key_hash).hex(),
        ],
    }
    if token_address is not None:
        params["address"] = token_address
    return params


def _resolve_from_block(w3: Web3, from_block: int | str) -> int:
    """Resolve ``from_block`` to an integer."""
    if isinstance(from_block, int):
        return from_block
    return w3.eth.block_number


async def _async_resolve_from_block(w3: AsyncWeb3, from_block: int | str) -> int:
    if isinstance(from_block, int):
        return from_block
    return await w3.eth.block_number


# ---------------------------------------------------------------------------
# Sync watcher
# ---------------------------------------------------------------------------


class SRC20EventWatcher:
    """Polling-based SRC20 event watcher (sync, runs in a background thread).

    Use the factory functions :func:`watch_src20_events` or
    :func:`watch_src20_events_with_key` to create instances.

    Example::

        watcher = watch_src20_events_with_key(
            w3,
            viewing_key=my_key,
            token_address="0x...",
            on_transfer=lambda log: print(log.decrypted_amount),
        )
        time.sleep(60)
        watcher.stop()

    Or as a context manager::

        with watch_src20_events_with_key(...) as watcher:
            time.sleep(60)
    """

    def __init__(
        self,
        w3: Web3,
        aes_key: Bytes32,
        *,
        token_address: ChecksumAddress | None = None,
        on_transfer: TransferCallback | None = None,
        on_approval: ApprovalCallback | None = None,
        on_error: ErrorCallback | None = None,
        poll_interval: float = 2.0,
        from_block: int | str = "latest",
    ) -> None:
        self._w3 = w3
        self._aes_key = aes_key
        self._encrypt_key_hash = compute_key_hash(aes_key)
        self._token_address = token_address
        self._on_transfer = on_transfer
        self._on_approval = on_approval
        self._on_error = on_error
        self._poll_interval = poll_interval
        self._initial_from_block = from_block

        self._stop_event = threading.Event()
        self._thread: threading.Thread | None = None

    # -- lifecycle ----------------------------------------------------------

    def start(self) -> None:
        """Start the background polling thread."""
        if self._thread is not None:
            return
        self._stop_event.clear()
        self._thread = threading.Thread(
            target=self._poll_loop, daemon=True, name="src20-watcher"
        )
        self._thread.start()

    def stop(self) -> None:
        """Signal the polling thread to stop and wait for it to exit."""
        self._stop_event.set()
        if self._thread is not None:
            self._thread.join(timeout=self._poll_interval * 3)
            self._thread = None

    @property
    def is_running(self) -> bool:
        return self._thread is not None and self._thread.is_alive()

    def __enter__(self) -> SRC20EventWatcher:
        self.start()
        return self

    def __exit__(self, *exc: object) -> None:
        self.stop()

    # -- internal -----------------------------------------------------------

    def _poll_loop(self) -> None:
        current_block = _resolve_from_block(self._w3, self._initial_from_block)

        while not self._stop_event.is_set():
            try:
                latest = self._w3.eth.block_number
                if current_block > latest:
                    self._stop_event.wait(self._poll_interval)
                    continue

                params = _build_filter_params(
                    self._token_address,
                    self._encrypt_key_hash,
                    current_block,
                    latest,
                )
                logs = self._w3.eth.get_logs(params)

                for log in logs:
                    self._process_log(dict(log))

                current_block = latest + 1

            except Exception as exc:
                if self._on_error:
                    self._on_error(exc)
                else:
                    logger.debug("SRC20 watcher poll error: %s", exc)

            self._stop_event.wait(self._poll_interval)

    def _process_log(self, log: dict[str, Any]) -> None:
        try:
            decoded = _decode_log(log, self._aes_key)
        except Exception as exc:
            if self._on_error:
                self._on_error(exc)
            return

        if decoded is None:
            return
        if isinstance(decoded, DecryptedTransferLog) and self._on_transfer:
            self._on_transfer(decoded)
        elif isinstance(decoded, DecryptedApprovalLog) and self._on_approval:
            self._on_approval(decoded)


# ---------------------------------------------------------------------------
# Async watcher
# ---------------------------------------------------------------------------


class AsyncSRC20EventWatcher:
    """Polling-based SRC20 event watcher (async, runs as an ``asyncio.Task``).

    Use the factory functions :func:`async_watch_src20_events` or
    :func:`async_watch_src20_events_with_key` to create instances.
    """

    def __init__(
        self,
        w3: AsyncWeb3,
        aes_key: Bytes32,
        *,
        token_address: ChecksumAddress | None = None,
        on_transfer: AsyncTransferCallback | TransferCallback | None = None,
        on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
        on_error: AsyncErrorCallback | ErrorCallback | None = None,
        poll_interval: float = 2.0,
        from_block: int | str = "latest",
    ) -> None:
        self._w3 = w3
        self._aes_key = aes_key
        self._encrypt_key_hash = compute_key_hash(aes_key)
        self._token_address = token_address
        self._on_transfer = on_transfer
        self._on_approval = on_approval
        self._on_error = on_error
        self._poll_interval = poll_interval
        self._initial_from_block = from_block

        self._task: asyncio.Task[None] | None = None

    # -- lifecycle ----------------------------------------------------------

    async def start(self) -> None:
        """Start the async polling task."""
        if self._task is not None:
            return
        self._task = asyncio.create_task(self._poll_loop())

    async def stop(self) -> None:
        """Cancel the polling task and wait for it to finish."""
        if self._task is not None:
            self._task.cancel()
            with contextlib.suppress(asyncio.CancelledError):
                await self._task
            self._task = None

    @property
    def is_running(self) -> bool:
        return self._task is not None and not self._task.done()

    async def __aenter__(self) -> AsyncSRC20EventWatcher:
        await self.start()
        return self

    async def __aexit__(self, *exc: object) -> None:
        await self.stop()

    # -- internal -----------------------------------------------------------

    async def _poll_loop(self) -> None:
        current_block = await _async_resolve_from_block(
            self._w3, self._initial_from_block
        )

        while True:
            try:
                latest = await self._w3.eth.block_number
                if current_block > latest:
                    await asyncio.sleep(self._poll_interval)
                    continue

                params = _build_filter_params(
                    self._token_address,
                    self._encrypt_key_hash,
                    current_block,
                    latest,
                )
                logs = await self._w3.eth.get_logs(params)

                for log in logs:
                    await self._process_log(dict(log))

                current_block = latest + 1

            except asyncio.CancelledError:
                raise
            except Exception as exc:
                await self._call_error(exc)

            await asyncio.sleep(self._poll_interval)

    async def _process_log(self, log: dict[str, Any]) -> None:
        try:
            decoded = _decode_log(log, self._aes_key)
        except Exception as exc:
            await self._call_error(exc)
            return

        if decoded is None:
            return
        if isinstance(decoded, DecryptedTransferLog) and self._on_transfer:
            result = self._on_transfer(decoded)
            if asyncio.iscoroutine(result):
                await result
        elif isinstance(decoded, DecryptedApprovalLog) and self._on_approval:
            result = self._on_approval(decoded)
            if asyncio.iscoroutine(result):
                await result

    async def _call_error(self, exc: Exception) -> None:
        if self._on_error:
            result = self._on_error(exc)
            if asyncio.iscoroutine(result):
                await result
        else:
            logger.debug("SRC20 async watcher poll error: %s", exc)


# ---------------------------------------------------------------------------
# Factory functions
# ---------------------------------------------------------------------------


def watch_src20_events(
    w3: Web3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    token_address: ChecksumAddress | None = None,
    on_transfer: TransferCallback | None = None,
    on_approval: ApprovalCallback | None = None,
    on_error: ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> SRC20EventWatcher:
    """Watch SRC20 events for the connected wallet (sync).

    Fetches the caller's viewing key from the Directory contract
    via signed read, then starts polling for matching events.

    Args:
        w3: Sync ``Web3`` instance (with ``w3.seismic``).
        encryption: Encryption state.
        private_key: 32-byte signing key.
        token_address: SRC20 contract address to filter (``None`` = all).
        on_transfer: Callback for Transfer events.
        on_approval: Callback for Approval events.
        on_error: Callback for errors (decryption failures, RPC errors).
        poll_interval: Seconds between polls (default ``2.0``).
        from_block: Starting block number or ``"latest"`` (default).

    Returns:
        A started :class:`SRC20EventWatcher`.  Call ``.stop()`` to
        terminate, or use as a context manager.
    """
    aes_key = get_viewing_key(w3, encryption, private_key)
    watcher = SRC20EventWatcher(
        w3,
        aes_key,
        token_address=token_address,
        on_transfer=on_transfer,
        on_approval=on_approval,
        on_error=on_error,
        poll_interval=poll_interval,
        from_block=from_block,
    )
    watcher.start()
    return watcher


def watch_src20_events_with_key(
    w3: Web3,
    *,
    viewing_key: Bytes32,
    token_address: ChecksumAddress | None = None,
    on_transfer: TransferCallback | None = None,
    on_approval: ApprovalCallback | None = None,
    on_error: ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> SRC20EventWatcher:
    """Watch SRC20 events using an explicit viewing key (sync).

    Does **not** require a wallet or encryption state — only a plain
    ``Web3`` instance and the 32-byte viewing key.

    Args:
        w3: Sync ``Web3`` instance.
        viewing_key: 32-byte AES-256 viewing key.
        token_address: SRC20 contract address to filter (``None`` = all).
        on_transfer: Callback for Transfer events.
        on_approval: Callback for Approval events.
        on_error: Callback for errors.
        poll_interval: Seconds between polls (default ``2.0``).
        from_block: Starting block number or ``"latest"`` (default).

    Returns:
        A started :class:`SRC20EventWatcher`.
    """
    watcher = SRC20EventWatcher(
        w3,
        viewing_key,
        token_address=token_address,
        on_transfer=on_transfer,
        on_approval=on_approval,
        on_error=on_error,
        poll_interval=poll_interval,
        from_block=from_block,
    )
    watcher.start()
    return watcher


async def async_watch_src20_events(
    w3: AsyncWeb3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    token_address: ChecksumAddress | None = None,
    on_transfer: AsyncTransferCallback | TransferCallback | None = None,
    on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
    on_error: AsyncErrorCallback | ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> AsyncSRC20EventWatcher:
    """Watch SRC20 events for the connected wallet (async).

    Fetches the caller's viewing key from the Directory contract
    via signed read, then starts polling for matching events.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        encryption: Encryption state.
        private_key: 32-byte signing key.
        token_address: SRC20 contract address to filter (``None`` = all).
        on_transfer: Callback for Transfer events.
        on_approval: Callback for Approval events.
        on_error: Callback for errors.
        poll_interval: Seconds between polls (default ``2.0``).
        from_block: Starting block number or ``"latest"`` (default).

    Returns:
        A started :class:`AsyncSRC20EventWatcher`.
    """
    from seismic_web3.src20.directory import async_get_viewing_key

    aes_key = await async_get_viewing_key(w3, encryption, private_key)
    watcher = AsyncSRC20EventWatcher(
        w3,
        aes_key,
        token_address=token_address,
        on_transfer=on_transfer,
        on_approval=on_approval,
        on_error=on_error,
        poll_interval=poll_interval,
        from_block=from_block,
    )
    await watcher.start()
    return watcher


async def async_watch_src20_events_with_key(
    w3: AsyncWeb3,
    *,
    viewing_key: Bytes32,
    token_address: ChecksumAddress | None = None,
    on_transfer: AsyncTransferCallback | TransferCallback | None = None,
    on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
    on_error: AsyncErrorCallback | ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> AsyncSRC20EventWatcher:
    """Watch SRC20 events using an explicit viewing key (async).

    Does **not** require a wallet or encryption state.

    Args:
        w3: Async ``AsyncWeb3`` instance.
        viewing_key: 32-byte AES-256 viewing key.
        token_address: SRC20 contract address to filter (``None`` = all).
        on_transfer: Callback for Transfer events.
        on_approval: Callback for Approval events.
        on_error: Callback for errors.
        poll_interval: Seconds between polls (default ``2.0``).
        from_block: Starting block number or ``"latest"`` (default).

    Returns:
        A started :class:`AsyncSRC20EventWatcher`.
    """
    watcher = AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        token_address=token_address,
        on_transfer=on_transfer,
        on_approval=on_approval,
        on_error=on_error,
        poll_interval=poll_interval,
        from_block=from_block,
    )
    await watcher.start()
    return watcher
