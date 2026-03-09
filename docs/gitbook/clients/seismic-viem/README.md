---
description: >-
  TypeScript client library for Seismic, composing with viem to add shielded
  transactions, encrypted calldata, and signed reads.
icon: plug
---

# Seismic Viem

Low-level TypeScript SDK for [Seismic](https://seismic.systems), built on [viem](https://viem.sh/) 2.x. Provides shielded public and wallet clients, encrypted contract interactions, signed reads, chain configs, and precompile access. This is the foundation that [seismic-react](../seismic-react/) builds upon.

```bash
npm install seismic-viem viem
```

## Quick Example

```typescript
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { createShieldedWalletClient, seismicTestnet } from "seismic-viem";

const client = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});

// Shielded write — calldata encrypted automatically
const hash = await client.writeContract({
  address: "0x...",
  abi: myContractAbi,
  functionName: "transfer",
  args: ["0x...", 100n],
});

// Signed read — proves caller identity
const balance = await client.readContract({
  address: "0x...",
  abi: myContractAbi,
  functionName: "balanceOf",
  args: ["0x..."],
  account: client.account,
});
```

## Architecture

```
seismic-viem
├── Client Layer
│   ├── createShieldedPublicClient  — read-only, TEE key, precompiles
│   └── createShieldedWalletClient  — full capabilities, encryption pipeline
├── Contract Layer
│   ├── getShieldedContract          — .read / .write / .tread / .twrite / .dwrite
│   ├── shieldedWriteContract        — standalone encrypted write
│   └── signedReadContract           — standalone signed read
├── Chain Configs
│   ├── seismicTestnet               — public testnet (chain ID 5124)
│   ├── sanvil                       — local dev (chain ID 31337)
│   └── createSeismicDevnet          — custom chain factory
├── Encryption
│   ├── getEncryption                — ECDH key exchange → AES key
│   └── AesGcmCrypto                 — encrypt/decrypt calldata
└── Precompiles
    ├── rng                          — random number generation
    ├── ecdh                         — key exchange
    ├── aesGcmEncrypt / Decrypt      — on-chain encryption
    ├── hkdf                         — key derivation
    └── secp256k1Sign                — signature generation
```

## Documentation Navigation

### Getting Started

| Section                             | Description                                           |
| ----------------------------------- | ----------------------------------------------------- |
| **[Installation](installation.md)** | Install seismic-viem, configure viem peer dependency  |
| **[Chains](chains.md)**             | Pre-configured chain definitions for Seismic networks |

### Client Reference

| Section                    | Description                                                           |
| -------------------------- | --------------------------------------------------------------------- |
| **Shielded Public Client** | Read-only client with TEE key exchange and precompile access          |
| **Shielded Wallet Client** | Full-featured client with encryption pipeline and transaction signing |

### Contract Interaction

| Section               | Description                                                                 |
| --------------------- | --------------------------------------------------------------------------- |
| **Contract Instance** | `getShieldedContract` for `.read`, `.write`, `.tread`, `.twrite`, `.dwrite` |
| **Shielded Writes**   | Encrypt calldata and send Seismic transactions (type 0x4A)                  |
| **Signed Reads**      | Prove caller identity in `eth_call` via `seismic_call`                      |

### Infrastructure

| Section                 | Description                                                     |
| ----------------------- | --------------------------------------------------------------- |
| **[Chains](chains.md)** | `seismicTestnet`, `sanvil`, `createSeismicDevnet` chain configs |
| **Encryption**          | ECDH key exchange, AES-GCM calldata encryption                  |
| **Precompiles**         | RNG, ECDH, AES-GCM, HKDF, secp256k1 signing precompile bindings |

## Features

- **Shielded Transactions** -- Encrypt calldata with TEE public key via AES-GCM before sending
- **Signed Reads** -- Prove identity in `eth_call` with signed read requests
- **Two Client Types** -- `createShieldedPublicClient` (read-only) and `createShieldedWalletClient` (full capabilities)
- **Contract Abstraction** -- `getShieldedContract` provides `.read`, `.write`, `.tread`, `.twrite`, and `.dwrite` methods
- **Automatic Encryption Pipeline** -- Calldata encryption handled transparently in the client layer
- **Type 0x4A Transactions** -- Native support for Seismic transaction type with custom chain formatters
- **Precompile Bindings** -- TypeScript wrappers for all Seismic precompiles (RNG, ECDH, AES-GCM, HKDF, secp256k1)
- **Chain Configs** -- Pre-configured chain definitions for testnet, local dev, and custom devnets
- **Full viem Compatibility** -- All standard viem client methods work unchanged

## Quick Links

### By Task

- **Create a wallet client** -> Shielded Wallet Client
- **Create a read-only client** -> Shielded Public Client
- **Interact with a contract** -> Contract Instance
- **Send an encrypted transaction** -> Shielded Writes
- **Read with identity proof** -> Signed Reads
- **Connect to a network** -> [Chains](chains.md)
- **Understand calldata encryption** -> Encryption
- **Install the package** -> [Installation](installation.md)

### By Component

- **Client types** -> Shielded Public Client, Shielded Wallet Client
- **Contract interaction** -> Contract Instance, Shielded Writes, Signed Reads
- **Chains** -> [Chains](chains.md) (`seismicTestnet`, `sanvil`, `createSeismicDevnet`)
- **Encryption** -> Encryption (`getEncryption`, `AesGcmCrypto`)
- **Precompiles** -> Precompiles (`rng`, `ecdh`, `aesGcmEncrypt`, `hkdf`, `secp256k1Sign`)

## Comparison with seismic-react

| Aspect               | seismic-viem                          | seismic-react                                 |
| -------------------- | ------------------------------------- | --------------------------------------------- |
| **Level**            | Low-level SDK                         | React hooks layer                             |
| **Framework**        | Framework-agnostic TypeScript         | React 18+                                     |
| **Foundation**       | Built directly on viem 2.x            | Built on seismic-viem + wagmi                 |
| **Client creation**  | Manual (`createShieldedWalletClient`) | Automatic via `ShieldedProvider`              |
| **State management** | Manual                                | React hooks (`useShieldedWalletClient`, etc.) |
| **Use when**         | Server-side, scripts, non-React apps  | React applications                            |

{% hint style="info" %}
If you are building a React application, consider using [seismic-react](../seismic-react/) which provides React hooks on top of seismic-viem. For Node.js scripts, server-side code, or non-React frontends, use seismic-viem directly.
{% endhint %}

## Next Steps

1. **[Install seismic-viem](installation.md)** -- Add the package to your project
2. **[Configure a chain](chains.md)** -- Connect to testnet or local dev
3. **Create a shielded wallet client** -- Connect with full capabilities
4. **Understand encryption** -- Learn how calldata encryption works
5. **Interact with contracts** -- Use `getShieldedContract` for reads and writes
6. **Explore precompiles** -- Access on-chain RNG, ECDH, and AES-GCM
