# Seismic Contracts

This repository contains solidity smart contracts and libraries designed for the [Seismic](https://seismic.systems) blockchain, a privacy-preserving EVM-compatible network. These contracts demonstrate how to leverage Seismic's unique features—**shielded storage** and **cryptographic precompiles**—to build applications that are impossible on standard Ethereum.

### Build, Test, Lint

This project uses [Seismic-Foundry](https://github.com/SeismicSystems/seismic-foundry), which is needed to compile any contracts with shielded types or seismic precompiles.

We recommend using our [Makefile](./Makefile) to run common tasks, as these will also be used in CI:
```bash
make build
make test
make fmt
...
```

## Project Structure

```
seismic-contracts/
├── src/
│   ├── directory/
│   │   ├── Directory.sol          # Key management contract
│   │   └── IDirectory.sol
│   ├── intelligence/
│   │   ├── Intelligence.sol       # Multi-provider encryption
│   │   └── IIntelligence.sol
│   └── seismic-std-lib/
│       ├── DepositContract.sol    # Eth2 staking deposits
│       ├── ProtocolParams.sol     # Protocol configuration
│       ├── session-keys/
│       │   ├── ShieldedDelegationAccount.sol
│       │   └── interfaces/
│       └── utils/
│           ├── MultiSend.sol      # Batch execution (from Safe)
│           ├── EIP7702Utils.sol   # Signature verification
│           ├── precompiles/
│               └── CryptoUtils.sol
├── test/                          # Foundry tests
└── artifacts/                     # Compiled contracts
```

### Artifacts

The `artifacts/` directory contains compiled contract artifacts, including ABIs and bytecode. These are used for deployment and interaction with the contracts.

These artifacts are currently generated manually using `make sync-artifacts`.

TODO: we need to figure out a way to version these and make it more explicit which of these are deployed on each network, and at which block (or genesis).

## Contracts

### Directory (`src/directory/Directory.sol`)

A key management and encryption service that allows users to register encryption keys and enables others to encrypt messages to them.

#### How It Works

1. Users call `setKey(suint256 _key)` to register their 256-bit encryption key
2. Anyone can call `encrypt(address to, bytes plaintext)` to encrypt a message to a registered user
3. The recipient calls `decrypt(bytes encryptedData)` to decrypt messages sent to them

#### Seismic Features Used

- **Shielded Storage**: Keys are stored as `mapping(address => suint256)`, making them invisible to external observers while remaining usable within the contract
- **AES Precompiles**: Encryption (`0x66`) and decryption (`0x67`) are performed via precompiled contracts

#### Comparison to Ethereum

On standard Ethereum, this contract would be **impossible to implement securely**:
- All storage is publicly readable, so encryption keys stored on-chain would be exposed
- Users would need to manage keys off-chain, defeating the purpose of a decentralized directory
- Any encryption scheme using on-chain keys would be trivially breakable

---

### Intelligence (`src/intelligence/Intelligence.sol`)

A multi-provider encryption orchestration contract that encrypts data to multiple registered providers simultaneously.

#### How It Works

1. Owner adds providers via `addProvider(address)`
2. Providers register their encryption keys in the Directory contract
3. Anyone calls `encryptToProviders(bytes plaintext)` to encrypt data to all providers at once
4. Returns an array of key hashes and corresponding encrypted data for each provider

#### Seismic Features Used

- **Directory Integration**: Leverages the Directory contract (at genesis address `0x1000000000000000000000000000000000000004`) for key management and encryption
- **Inherits Shielded Storage**: Provider keys remain confidential through the Directory's `suint256` storage

#### Comparison to Ethereum

On Ethereum, broadcasting encrypted data to multiple parties would require:
- Off-chain key exchange with each provider
- Client-side encryption before submission
- No guarantee that providers have registered valid keys

With Seismic, the entire workflow happens trustlessly on-chain with cryptographic guarantees.

---

### ShieldedDelegationAccount (`src/seismic-std-lib/session-keys/ShieldedDelegationAccount.sol`)

An experimental [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702) delegation contract that supports session keys with spend limits and encrypted transaction execution.

> **WARNING**: This contract is experimental and has not been audited.

#### How It Works

1. **Initialization**: Owner calls `setAESKey()` to generate a random AES key (stored as `suint256`)
2. **Key Authorization**: Owner authorizes session keys via `authorizeKey(keyType, publicKey, expiry, limitWei)`
   - Supports P256, WebAuthnP256, and Secp256k1 key types
   - Each key has an expiry timestamp and optional spend limit
3. **Encrypted Execution**:
   - Caller encrypts transaction data using `encrypt(plaintext)` (view function)
   - Caller signs the encrypted payload using EIP-712 typed data
   - Caller submits via `execute(nonce, encryptedCalls, signature, keyIndex)`
   - Contract verifies signature, checks limits, decrypts, and executes

#### Seismic Features Used

- **Shielded Storage**: The AES key is stored as `suint256 aesKey`, keeping it confidential
- **RNG Precompile** (`0x64`): Generates random AES key and nonces
- **AES Precompiles** (`0x66`, `0x67`): Encrypts/decrypts transaction calldata

#### Comparison to Ethereum

On Ethereum, session keys exist but with significant limitations:
- Transaction contents are always visible in the mempool and on-chain
- Spend limits can only restrict ETH value, not hide transaction details
- Attackers can front-run or analyze transaction patterns

With Seismic's ShieldedDelegationAccount:
- Transaction calldata is encrypted—observers cannot see what actions are being taken
- The AES key never leaves the enclave, so only the account can decrypt
- Combined with Seismic's encrypted transaction type (`0x4a`), the entire flow is private
