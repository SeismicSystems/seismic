"""Seismic cryptographic primitives.

This package provides the ECDH key exchange, AES-GCM encryption,
HKDF key derivation, and secp256k1 key utilities used to build
``TxSeismic`` encrypted transactions.

Security notes:
    - ``coincurve`` wraps Bitcoin Core's libsecp256k1 (battle-tested).
    - ``cryptography`` wraps OpenSSL (FIPS 140-2 validated backend).
    - Neither is a pure-Python implementation.
"""
