---
icon: gear
---

# How Seismic Works

Seismic adds on-chain privacy to the EVM through three layers: a modified Solidity compiler, a network of TEE-secured nodes, and a shielded storage model. This page explains how they fit together.

## Three pillars

| Layer                                 | What it does                                                                                     |
| ------------------------------------- | ------------------------------------------------------------------------------------------------ |
| **Seismic Solidity compiler**         | Adds shielded types (`suint`, `sint`, `sbool`, `saddress`) that compile to privacy-aware opcodes |
| **Seismic network**                   | EVM-compatible L1 where nodes run inside Trusted Execution Environments (TEEs)                   |
| **Shielded storage (FlaggedStorage)** | Storage model where each slot carries an `is_private` flag, enforced at the opcode level         |

## The Seismic Solidity compiler

The Seismic compiler (`ssolc`) is a fork of `solc` that understands shielded types. When you write:

```solidity
suint256 balance = suint256(100);
```

the compiler emits `CSTORE` (opcode `0xB1`) instead of `SSTORE` to write the value, and `CLOAD` (opcode `0xB0`) instead of `SLOAD` to read it. These opcodes tell the EVM to treat the storage slot as private.

The supported shielded types are:

- **`suint`** / **`sint`** -- shielded integers (all standard bit widths: `suint8` through `suint256`)
- **`sbool`** -- shielded boolean
- **`saddress`** -- shielded address

Arithmetic, comparisons, and assignments work exactly as they do with regular Solidity types. The compiler handles routing to the correct opcodes. You do not need to call any special APIs or change your contract logic.

## The Seismic network

Seismic is an EVM-compatible L1 blockchain. Nodes run inside **Trusted Execution Environments** (TEEs) powered by Intel TDX. The TEE creates a hardware-enforced enclave: code and data inside the enclave cannot be observed or tampered with by the host operating system or the node operator.

Key network properties:

- **Block time:** \~600ms
- **Finality:** 1 block
- **Transaction types:** All standard Ethereum types (Legacy, EIP-1559, EIP-2930, EIP-4844, EIP-7702) plus the Seismic transaction type `0x4A`

### The Seismic transaction (type 0x4A)

Standard Ethereum transactions send calldata in plaintext. The Seismic transaction type encrypts calldata before it leaves the user's machine.

The encryption flow:

1. The client calls `seismic_getTeePublicKey` on the RPC to fetch the network's TEE public key.
2. The client performs ECDH key agreement between the user's private key and the TEE public key to derive a shared secret.
3. The calldata is encrypted using AEAD (authenticated encryption with associated data).
4. The encrypted transaction is broadcast to the network.
5. Inside the TEE, the node decrypts the calldata, executes the transaction, and writes results to shielded storage.

At no point is the plaintext calldata visible outside the TEE -- not in the mempool, not in block data, not in transaction traces.

## Shielded storage (FlaggedStorage)

Seismic extends the EVM storage model with **FlaggedStorage**. Each storage slot is a tuple:

```
(value, is_private)
```

When a contract uses `CSTORE` to write a value, the `is_private` flag is set to `true`. This flag has two effects:

- **`eth_getStorageAt` calls fail** for private slots. External observers cannot read shielded data through the standard RPC.
- **Only `CLOAD` can read private slots.** The standard `SLOAD` opcode cannot access them. This is enforced at the EVM level.

The compiler manages this automatically. When you declare a variable as `suint256`, the compiler emits `CLOAD`/`CSTORE`. When you declare it as `uint256`, the compiler emits `SLOAD`/`SSTORE`. You do not interact with FlaggedStorage directly.

## End-to-end walkthrough

Here is what happens when a user calls `transfer()` on an SRC20 contract with shielded balances:

```solidity
mapping(address => suint256) balanceOf;

function transfer(address to, suint256 amount) public {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

**Step 1: User encrypts calldata.** The client library (e.g., `seismic-viem`) fetches the TEE public key, derives a shared secret via ECDH, and encrypts the function arguments (`to` and `amount`) using AEAD. The encrypted payload is wrapped in a type `0x4A` transaction.

**Step 2: Transaction enters the mempool.** The calldata is encrypted. Observers can see that a transaction was submitted to the SRC20 contract, but the recipient address and amount are not readable.

**Step 3: Node decrypts inside the TEE.** The Seismic node, running inside Intel TDX, decrypts the calldata using its TEE private key. The plaintext arguments are now available only within the enclave.

**Step 4: EVM executes with CSTORE.** The EVM processes the `transfer()` function. Reads from `balanceOf` use `CLOAD`. Writes to `balanceOf` use `CSTORE`. The storage slots are updated with `is_private = true`.

**Step 5: Storage is updated.** The new balances are written to FlaggedStorage. Each balance slot has its `is_private` flag set.

**Step 6: Observers see `0x000`.** Anyone querying the contract state, reading transaction traces, or inspecting block data sees `0x000` in place of all shielded values -- the balances, the transfer amount, and any intermediate computation involving shielded types.

## Precompiles

Seismic adds six precompiled contracts to the EVM, giving smart contracts access to cryptographic primitives that would be prohibitively expensive to implement in Solidity:

| Address | Name            | Purpose                                                                                     |
| ------- | --------------- | ------------------------------------------------------------------------------------------- |
| `0x64`  | RNG             | Securely generate a random number                                                           |
| `0x65`  | ECDH            | Elliptic Curve Diffie-Hellman -- derive a shared secret from a public key and a private key |
| `0x66`  | AES-GCM Encrypt | Encrypt data with AES-GCM                                                                   |
| `0x67`  | AES-GCM Decrypt | Decrypt data with AES-GCM                                                                   |
| `0x68`  | HKDF            | Derive cryptographic keys from a parent key                                                 |
| `0x69`  | secp256k1 Sign  | Sign a message with a secret key                                                            |

These precompiles enable contracts to perform on-chain encryption, key derivation, and random number generation without relying on external oracles or off-chain computation.

## System architecture

The system is composed of three components that work together to provide confidential smart contract execution:

| Component        | Role                                                               |
| ---------------- | ------------------------------------------------------------------ |
| **Seismic Node** | Transaction processing, state management, RPC -- runs inside a TEE |
| **Summit**       | Consensus and block production                                     |
| **Enclave**      | TEE operations: key management, encryption/decryption, attestation |

All three components are designed so that private data is only ever accessible inside the Trusted Execution Environment. No plaintext shielded data leaves the TEE boundary at any point in the pipeline.

### Seismic node

The Seismic node is a fork of [reth](https://github.com/paradigmxyz/reth) (the Rust Ethereum execution client). It handles:

- **RPC**: Accepts incoming transactions and read requests. Serves responses to clients, redacting shielded data from public queries.
- **EVM execution**: Runs a modified EVM that supports `CLOAD`/`CSTORE` opcodes and the six Seismic [precompiles](../reference/precompiles.md). This is built on a forked version of `revm`.
- **State management**: Maintains the world state using FlaggedStorage, where each storage slot is tagged as public or private.
- **Transaction pool**: Receives both standard Ethereum transactions and Seismic transactions (type `0x4A`). Encrypted calldata in Seismic transactions is decrypted inside the TEE before execution.

The entire node process runs inside an Intel TDX Trusted Execution Environment. This means the node operator cannot inspect memory, attach debuggers, or extract keys from the running process.

### Fork chain

The Seismic execution stack is built on a chain of forks from the Ethereum Rust ecosystem:

```
alloy-core  -->  revm  -->  reth
    |               |          |
seismic-alloy  seismic-revm  seismic-reth
```

- **seismic-alloy** (fork of alloy-core): Adds the Seismic transaction type (`0x4A`) and FlaggedStorage primitives to the core types.
- **seismic-revm** (fork of revm): Implements `CLOAD`/`CSTORE` opcodes and the Seismic precompiles in the EVM interpreter. Enforces the FlaggedStorage access rules at the opcode level.
- **seismic-reth** (fork of reth): Integrates the modified EVM into the full node, adding Enclave communication, modified RPC behavior (redacting private data), and TEE attestation.

FlaggedStorage flows through every layer of this stack. At the alloy level, it is a type definition. At the revm level, it governs opcode behavior. At the reth level, it determines what the RPC returns to callers.

### Summit (consensus)

Summit is Seismic's consensus layer, built on [Commonware](https://github.com/commonwarexyz/monorepo) primitives. It handles:

- **Block production**: Ordering transactions into blocks with \~600ms block times.
- **Consensus**: Reaching agreement among validators on the canonical chain.
- **Finality**: Single-block finality -- once a block is produced and agreed upon, it is final.

Summit communicates with the Seismic node to receive transactions from the mempool and to deliver finalized blocks for execution.

### Enclave

The Enclave component manages all TEE-related operations. It is the trust anchor of the system.

**Key management:**

- **Genesis node**: When the network starts, the genesis node generates a root key inside the TEE. This key never leaves the enclave.
- **Peer nodes**: When a new node joins the network, it must pass remote attestation before receiving the root key. The existing nodes verify that the new node is running identical, approved code inside a genuine TEE.
- **Encryption secret key**: The root key is used to derive the network's encryption secret key. This key is used to decrypt the calldata of Seismic transactions (type `0x4A`).

**Attestation:**

Remote attestation is the process by which one TEE proves to another that it is running approved code on genuine hardware. In Seismic:

1. A new node generates an attestation report inside its TEE.
2. The report is sent to existing nodes for verification.
3. Existing nodes check that the report was generated by genuine Intel TDX hardware and that the code measurement matches the approved build.
4. Only after successful verification does the new node receive the root key.

This ensures that every node in the network is running the same software and that no node can be modified to leak private data.

## TEE guarantees

The TEE (Trusted Execution Environment) is the foundation of Seismic's privacy model. Intel TDX provides the following guarantees:

### Code integrity

Remote attestation ensures that all nodes in the network are running identical, approved code. A node cannot be modified to log private data, skip encryption, or export keys.

### Memory isolation

The TEE creates a hardware-enforced boundary around the node's memory. The host operating system, hypervisor, and node operator cannot read or write to the enclave's memory space.

### Key protection

Cryptographic keys (the root key, encryption secret key, and any derived keys) are generated inside the TEE and never leave it. There is no API to export keys from the enclave. Even if the node operator has full root access to the host machine, they cannot extract the keys.

### What TEEs do not protect against

- **Side-channel attacks**: While Intel TDX mitigates many known side-channel attacks, this is an active area of research. Seismic's design minimizes the attack surface, but hardware-level side channels remain a theoretical concern.
- **Bugs in the node software**: If the approved node code has a bug that leaks private data through a public channel (e.g., writing shielded values to public storage), the TEE will faithfully execute that buggy code. This is why the code is open-source and subject to audit.
- **Transaction metadata**: The TEE protects calldata and storage values, but metadata such as sender address, gas usage, and the target contract address remain visible on-chain.
