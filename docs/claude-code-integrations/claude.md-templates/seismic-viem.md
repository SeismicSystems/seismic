---
description: CLAUDE.md template for TypeScript dapp development with seismic-viem
icon: js
---

# Seismic Viem

Use this template when your project uses `seismic-viem` to interact with Seismic contracts from TypeScript or JavaScript. This is the most common client SDK for dapp backends, scripts, and CLI tools.

## The template

Copy the entire block below and save it as `CLAUDE.md` in your project root.

````markdown
# [Your Project Name]

## Seismic Overview

Seismic is an EVM-compatible L1 with on-chain privacy. Nodes run inside TEEs (Intel TDX). The Solidity compiler adds shielded types (`suint256`, `saddress`, `sbool`) that are invisible outside the TEE. Client libraries handle transaction encryption and signed reads automatically.

## Key Concepts

- **Shielded types**: `suint256`, `saddress`, `sbool` — on-chain private state, only readable via signed reads
- **TxSeismic (type 0x4A)**: Encrypts calldata before broadcast. The shielded wallet client handles this automatically.
- **Signed reads**: `eth_call` zeroes `msg.sender` on Seismic. For view functions that check `msg.sender`, use `contract.read.*` from a shielded contract instance — it sends a signed read automatically.
- **Encryption pubkeys**: 33-byte compressed secp256k1 public keys. The SDK fetches the TEE public key and derives a shared AES-GCM key via ECDH.
- **Legacy gas**: Seismic transactions use `gas_price` + `gas_limit` (legacy style), NOT EIP-1559 (`maxFeePerGas`/`maxPriorityFeePerGas`).

## SDK: seismic-viem

### Install

```bash
npm install seismic-viem
# or
bun add seismic-viem
```

### Key exports

```typescript
import {
  createShieldedWalletClient,
  createShieldedPublicClient,
  getShieldedContract,
  seismicDevnet, // Chain definition (chainId: 5124)
} from "seismic-viem";
```

## Core Patterns

### Create a shielded wallet client

```typescript
import { createShieldedWalletClient, seismicDevnet } from "seismic-viem";
import { http } from "viem";

const walletClient = await createShieldedWalletClient({
  chain: seismicDevnet,
  transport: http("https://node-2.seismicdev.net/rpc"),
  privateKey: "0xYOUR_PRIVATE_KEY",
});
```

### Create a shielded public client (read-only)

```typescript
import { createShieldedPublicClient, seismicDevnet } from "seismic-viem";
import { http } from "viem";

const publicClient = await createShieldedPublicClient({
  chain: seismicDevnet,
  transport: http("https://node-2.seismicdev.net/rpc"),
});
```

### Get a shielded contract instance

```typescript
import { getShieldedContract } from "seismic-viem";

const contract = getShieldedContract({
  abi: myContractAbi,
  address: "0xCONTRACT_ADDRESS",
  client: walletClient,
});
```

### Shielded write (encrypted transaction)

```typescript
// Sends a type 0x4A transaction with encrypted calldata
const hash = await contract.write.transfer([recipientAddress, amount], {
  gas: 500_000n,
});
```

### Signed read

```typescript
// Sends a signed eth_call so msg.sender is set correctly
const balance = await contract.read.getBalance([userAddress]);
```

### Wait for transaction receipt

```typescript
import { createPublicClient, http } from "viem";

// Use a standard viem public client for receipt watching
const publicClient = createPublicClient({
  chain: seismicDevnet,
  transport: http("https://node-2.seismicdev.net/rpc"),
});

const receipt = await publicClient.waitForTransactionReceipt({ hash });
```

## Common Mistakes

1. **Using standard viem `createWalletClient`** — This creates a regular Ethereum client. It won't encrypt calldata or handle signed reads. Always use `createShieldedWalletClient`.
2. **Using EIP-1559 gas params** — Seismic transactions use legacy gas (`gas` field). Do NOT pass `maxFeePerGas` or `maxPriorityFeePerGas`.
3. **Calling view functions with `publicClient.readContract()`** — If the contract function checks `msg.sender`, a standard read will fail (sender is zeroed). Use `contract.read.*` from a shielded contract instance instead.
4. **Forgetting `await` on client creation** — `createShieldedWalletClient` is async (it fetches the TEE public key). Missing `await` gives you a Promise, not a client.
5. **Wrong import path** — The package is `seismic-viem`, not `@seismic/viem` or `viem/seismic`.

## Networks

| Network        | Chain ID | RPC URL                             | Chain export    |
| -------------- | -------- | ----------------------------------- | --------------- |
| Devnet         | 5124     | `https://node-2.seismicdev.net/rpc` | `seismicDevnet` |
| Devnet (WS)    | 5124     | `wss://node-2.seismicdev.net/ws`    | `seismicDevnet` |
| Local (sanvil) | 31337    | `http://127.0.0.1:8545`             | —               |

Faucet: https://faucet-2.seismicdev.net/

## Links

- [seismic-viem Installation](https://docs.seismic.systems/client-libraries/seismic-viem/installation)
- [Shielded Wallet Client](https://docs.seismic.systems/client-libraries/seismic-viem/shielded-wallet-client)
- [Contract Instance](https://docs.seismic.systems/client-libraries/seismic-viem/contract-instance)
- [Signed Reads](https://docs.seismic.systems/client-libraries/seismic-viem/signed-reads)
- [GitHub: seismic-client](https://github.com/SeismicSystems/seismic-client)
````

## What this teaches Claude

- **Correct import paths and function names** — Claude will use `createShieldedWalletClient` and `getShieldedContract` instead of guessing
- **Async client creation** — Claude will `await` client construction since it involves TEE key exchange
- **Signed reads vs. plain reads** — Claude will use the contract instance's `.read` namespace for shielded data
- **Gas model** — Claude will use legacy gas parameters instead of EIP-1559

## Customizing

After pasting the template:

- Replace `[Your Project Name]` with your project name
- Add your contract ABIs and addresses
- Add your ABI import paths (e.g., `import abi from './abi/MyContract.json'`)
- If you have multiple contracts, list them under a `## Contracts` section
