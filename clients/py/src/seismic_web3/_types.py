"""Primitive byte types for the Seismic SDK.

All types subclass ``HexBytes`` (itself a ``bytes`` subclass), so they:
- Accept ``"0x…"`` hex strings, raw ``bytes``, or ``int`` on construction.
- Pass ``isinstance(x, bytes)`` — compatible with every web3.py API.
- Are immutable, hashable, and have a ``.to_0x_hex()`` method.

Fixed-size types validate length at construction time and raise
``ValueError`` on mismatch.
"""

from __future__ import annotations

from typing import ClassVar, cast

from hexbytes import HexBytes


class _SizedHexBytes(HexBytes):
    """Base class for fixed-size byte values.

    Subclasses **must** set the ``_size`` class variable to the expected
    byte length.  Construction from any input accepted by ``HexBytes``
    (hex strings, raw bytes, ints) is supported; a ``ValueError`` is
    raised when the resulting length does not match ``_size``.
    """

    _size: ClassVar[int]

    def __new__(cls, val: bytes | str | int) -> _SizedHexBytes:
        """Create a new fixed-size byte value.

        Args:
            val: Hex string (``"0x…"``), raw ``bytes``, or ``int``.

        Returns:
            Validated instance whose ``len()`` equals ``cls._size``.

        Raises:
            ValueError: If the decoded length does not match ``_size``.
        """
        obj = cast("_SizedHexBytes", super().__new__(cls, val))
        if len(obj) != cls._size:
            raise ValueError(
                f"{cls.__name__}: expected {cls._size} bytes, got {len(obj)}"
            )
        return obj


class Bytes32(_SizedHexBytes):
    """Exactly 32 bytes — used for hashes, AES keys, and similar values."""

    _size = 32


class PrivateKey(Bytes32):
    """32-byte secp256k1 private key."""

    @staticmethod
    def from_hex_str(hex_string: str) -> PrivateKey:
        """Create a ``PrivateKey`` from a hex string, with or without ``0x``.

        Shorthand for ``PrivateKey(hex_to_bytes(hex_string))``.

        Args:
            hex_string: Hex-encoded private key, with or without a
                ``"0x"`` prefix.

        Returns:
            A new ``PrivateKey`` instance.

        Examples::

            >>> pk = PrivateKey.from_hex_str("0xac0974bec...")
            >>> pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
        """
        return PrivateKey(hex_to_bytes(hex_string))


class CompressedPublicKey(_SizedHexBytes):
    """33-byte compressed secp256k1 public key (``0x02`` / ``0x03`` prefix)."""

    _size = 33

    def __new__(cls, val: bytes | str | int) -> CompressedPublicKey:
        """Create a compressed public key, validating prefix byte.

        Args:
            val: Hex string, raw bytes, or int.

        Returns:
            Validated 33-byte compressed public key.

        Raises:
            ValueError: If length is not 33 or prefix byte is not
                ``0x02`` or ``0x03``.
        """
        obj = cast("CompressedPublicKey", super().__new__(cls, val))
        if obj[0] not in (0x02, 0x03):
            raise ValueError(
                "Compressed public key must start with 0x02 or 0x03, "
                f"got 0x{obj[0]:02x}"
            )
        return obj


class EncryptionNonce(_SizedHexBytes):
    """12-byte AES-GCM encryption nonce."""

    _size = 12


# ---------------------------------------------------------------------------
# Convenience aliases
# ---------------------------------------------------------------------------

#: Arbitrary-length hex data (calldata, return data, etc.).
#: Just use ``HexBytes`` directly — no length constraint.
HexData = HexBytes

#: Nonce values accepted by encryption functions.
#: Either an ``int`` (converted to 12-byte big-endian) or a raw
#: ``EncryptionNonce``.
EncryptionNonceInput = int | EncryptionNonce


# ---------------------------------------------------------------------------
# Utility functions
# ---------------------------------------------------------------------------


def hex_to_bytes(hex_string: str) -> bytes:
    """Convert a hex string to raw bytes, stripping an optional ``0x`` prefix.

    Convenience wrapper around ``bytes.fromhex`` that accepts both
    ``"0xabcd…"`` and ``"abcd…"`` formats.

    Args:
        hex_string: Hex-encoded string, with or without a ``"0x"`` prefix.

    Returns:
        Decoded raw bytes.

    Raises:
        ValueError: If *hex_string* contains non-hex characters or has
            odd length after prefix removal.

    Examples::

        >>> hex_to_bytes("0xdeadbeef")
        b'\\xde\\xad\\xbe\\xef'
        >>> hex_to_bytes("deadbeef")
        b'\\xde\\xad\\xbe\\xef'
    """
    return bytes.fromhex(hex_string.removeprefix("0x"))
