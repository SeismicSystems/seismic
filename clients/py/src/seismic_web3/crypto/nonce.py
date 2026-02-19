"""Encryption nonce generation.

Provides a function to generate random 12-byte AES-GCM nonces,
working around an RLP encoding limitation where nonces are treated
as unsigned integers (no leading zeros allowed).
"""

from __future__ import annotations

import os

from seismic_web3._types import EncryptionNonce


def random_encryption_nonce() -> EncryptionNonce:
    """Generate a random 12-byte AES-GCM encryption nonce.

    Re-samples if the first byte is zero, because the Seismic RLP
    codec treats nonces as unsigned integers (leading zeros get
    stripped, changing the effective length).  This happens ~1/256
    of the time.

    Returns:
        A 12-byte ``EncryptionNonce`` with a nonzero first byte.
    """
    while True:
        raw = os.urandom(12)
        if raw[0] != 0:
            return EncryptionNonce(raw)
