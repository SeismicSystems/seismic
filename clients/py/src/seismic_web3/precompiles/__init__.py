"""Mercury EVM precompile helpers.

Standalone sync and async wrappers for the 6 on-chain precompiles
at addresses ``0x64`` through ``0x69``.  Each function takes a
``Web3`` or ``AsyncWeb3`` instance and returns the decoded result.

No encryption state or private key is required -- these use plain
``eth_call``.
"""

from seismic_web3.precompiles.aes import (
    aes_gcm_decrypt,
    aes_gcm_encrypt,
    async_aes_gcm_decrypt,
    async_aes_gcm_encrypt,
)
from seismic_web3.precompiles.ecdh import async_ecdh, ecdh
from seismic_web3.precompiles.hkdf import async_hkdf, hkdf
from seismic_web3.precompiles.rng import async_rng, rng
from seismic_web3.precompiles.secp256k1 import async_secp256k1_sign, secp256k1_sign

__all__ = [
    "aes_gcm_decrypt",
    "aes_gcm_encrypt",
    "async_aes_gcm_decrypt",
    "async_aes_gcm_encrypt",
    "async_ecdh",
    "async_hkdf",
    "async_rng",
    "async_secp256k1_sign",
    "ecdh",
    "hkdf",
    "rng",
    "secp256k1_sign",
]
