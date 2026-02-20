"""secp256k1 signing precompile (address ``0x69``).

On-chain ECDSA signing using the secp256k1 curve.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from eth_abi import encode as abi_encode
from eth_hash.auto import keccak
from hexbytes import HexBytes

from seismic_web3._types import Bytes32, PrivateKey
from seismic_web3.precompiles._base import (
    Precompile,
    async_call_precompile,
    call_precompile,
)

if TYPE_CHECKING:
    from web3 import AsyncWeb3, Web3

SECP256K1_SIG_ADDRESS = "0x0000000000000000000000000000000000000069"
_SECP256K1_SIG_BASE_GAS = 3000


@dataclass(frozen=True)
class Secp256k1SigParams:
    """Parameters for on-chain secp256k1 signing.

    Attributes:
        sk: 32-byte private key.
        message_hash: 32-byte message hash to sign.
    """

    sk: PrivateKey
    message_hash: Bytes32


def _sig_gas_cost(_params: Secp256k1SigParams) -> int:
    return _SECP256K1_SIG_BASE_GAS


def _sig_encode(params: Secp256k1SigParams) -> bytes:
    return abi_encode(
        ["bytes32", "bytes32"],
        [bytes(params.sk), bytes(params.message_hash)],
    )


def _sig_decode(result: bytes) -> HexBytes:
    return HexBytes(result)


secp256k1_sig_precompile: Precompile[Secp256k1SigParams, HexBytes] = Precompile(
    address=SECP256K1_SIG_ADDRESS,
    gas_cost=_sig_gas_cost,
    encode_params=_sig_encode,
    decode_result=_sig_decode,
)


def _hash_message(message: str) -> Bytes32:
    """Ethereum personal_sign message hash (EIP-191).

    Equivalent to viem's ``hashMessage()``.
    """
    prefix = f"\x19Ethereum Signed Message:\n{len(message)}".encode()
    return Bytes32(keccak(prefix + message.encode()))


def secp256k1_sign(
    w3: Web3,
    *,
    sk: PrivateKey,
    message: str,
) -> HexBytes:
    """Sign a message on-chain using secp256k1 (sync).

    The message is hashed with the EIP-191 personal-sign prefix
    before being passed to the precompile.

    Args:
        w3: Sync ``Web3`` instance connected to a Seismic node.
        sk: 32-byte private key.
        message: Message string to sign.

    Returns:
        Signature bytes.
    """
    msg_hash = _hash_message(message)
    return call_precompile(
        w3, secp256k1_sig_precompile, Secp256k1SigParams(sk, msg_hash)
    )


async def async_secp256k1_sign(
    w3: AsyncWeb3,
    *,
    sk: PrivateKey,
    message: str,
) -> HexBytes:
    """Sign a message on-chain using secp256k1 (async)."""
    msg_hash = _hash_message(message)
    return await async_call_precompile(
        w3, secp256k1_sig_precompile, Secp256k1SigParams(sk, msg_hash)
    )
