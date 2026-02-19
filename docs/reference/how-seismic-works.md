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

- **Block time:** ~600ms
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
