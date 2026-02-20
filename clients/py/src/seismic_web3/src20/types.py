"""Data types for SRC20 event watching.

Frozen dataclasses for decoded event logs and callback type aliases.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress
    from hexbytes import HexBytes


@dataclass(frozen=True)
class DecryptedTransferLog:
    """Decoded SRC20 Transfer event with decrypted amount."""

    from_address: ChecksumAddress
    to_address: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int


@dataclass(frozen=True)
class DecryptedApprovalLog:
    """Decoded SRC20 Approval event with decrypted amount."""

    owner: ChecksumAddress
    spender: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int


# Callback type aliases
TransferCallback = Callable[[DecryptedTransferLog], None]
ApprovalCallback = Callable[[DecryptedApprovalLog], None]
ErrorCallback = Callable[[Exception], None]

AsyncTransferCallback = Callable[[DecryptedTransferLog], Awaitable[None]]
AsyncApprovalCallback = Callable[[DecryptedApprovalLog], Awaitable[None]]
AsyncErrorCallback = Callable[[Exception], Awaitable[None]]

# Union types for accept-either-style in factory functions
AnyTransferCallback = TransferCallback | AsyncTransferCallback
AnyApprovalCallback = ApprovalCallback | AsyncApprovalCallback
AnyErrorCallback = ErrorCallback | AsyncErrorCallback
