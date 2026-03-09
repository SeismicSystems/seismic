# Seismic Concepts Glossary <!-- omit in toc -->

- [Shielded State / FlaggedStorage](#shielded-state--flaggedstorage)
- [Shielded Types](#shielded-types)
- [Mercury Spec](#mercury-spec)
- [TxSeismic](#txseismic)
- [TEE Integration](#tee-integration)
- [SeismicHost](#seismichost)


## Shielded State / FlaggedStorage

Every storage slot is a tuple `(value, is_private)`, called `FlaggedStorage`. Private slots cannot be read via `eth_getStorageAt`/`eth_getProof` — they return 0. Inside the EVM, only `CLOAD`/`CSTORE` opcodes can access them - `SLOAD`/`SSTORE` will fail if `is_private` is true (meaning they have already been `CSTORE`d to). This allows contracts to keep secrets on-chain that must be accessed via contract logic rather than direct reads. Authentication/Authorization libraries such as OpenZeppelin's AccessControl can thus be used to gate READ access to private state, unlike in a standard EVM where all state is public.

```rust
struct FlaggedStorage {
    value: U256,
    is_private: bool,
}
```

This type flows through: alloy-core → revm (journal) → trie (merkle encoding) → reth (database, RPC).

## Shielded Types

The Seismic Solidity compiler adds `suint`, `sint`, `sbool`, `saddress` — confidential counterparts of standard Solidity types. They compile down to `CLOAD`/`CSTORE` instead of `SLOAD`/`SSTORE`. They otherwise behave exactly like their public counterparts in terms of arithmetic, ABI encoding, etc. This allows developers to easily write contracts with private state without needing to manage the `(value, is_private)` tuple manually.

See [language-and-vm.md](language-and-vm.md) for the full spec (storage behavior, restrictions, casting, arrays, best practices).

## Mercury Spec

Seismic's name for the modified EVM specification. Implemented primarily in seismic-revm. Adds:
- **Opcodes**: CLOAD (0xB0), CSTORE (0xB1)
- **Precompiles**: RNG (0x64), ECDH (0x65), AES-GCM Encrypt (0x66), AES-GCM Decrypt (0x67), HKDF (0x68), secp256k1 Sign (0x69)
- **FlaggedStorage semantics**
- **Access rules**: SLOAD on private slots returns 0/halts; CLOAD on public slots returns 0/halts

## TxSeismic

Transaction type ID `74` (0x4A). Standard EVM transaction with encrypted calldata. Encryption uses ECDH + AEAD. Additional fields:

```rust
struct TxSeismicElements {
    encryption_pubkey: PublicKey,  // 33-byte compressed secp256k1
    encryption_nonce: U96,        // AEAD nonce (12 bytes)
    message_version: u8,
    recent_block_hash: B256,      // prevents replay across forks
    expires_at_block: u64,        // expiration block number
    signed_read: bool,
}
```

Defined in seismic-alloy, consumed by seismic-evm, seismic-reth, and seismic-foundry.

## TEE Integration

Nodes run inside a Trusted Execution Environment. The TEE holds the decryption key. Users encrypt calldata with the network's public key (fetched via `seismic_getTeePublicKey` RPC). Only the TEE can decrypt and execute. Note that decryption is done by the RPC layer, so as to not diverge from the ABI standard.

## SeismicHost

Trait in seismic-revm extending the standard EVM Host with confidential storage operations. Implemented by the journal/state layer.
