---
description: Pre-configured chain definitions for Seismic networks
icon: link
---

# Chains

`seismic-viem` provides pre-configured viem `Chain` objects with Seismic transaction formatters. These chain definitions include the correct chain IDs, RPC endpoints, and custom formatters needed for Seismic's type 0x4A transactions.

## Import

```typescript
import {
  seismicTestnet,
  sanvil,
  localSeismicDevnet,
  createSeismicDevnet,
} from "seismic-viem";
```

## Available Chains

| Chain           | Export               | Chain ID | RPC URL                             | Description              |
| --------------- | -------------------- | -------- | ----------------------------------- | ------------------------ |
| Seismic Testnet | `seismicTestnet`     | 5124     | `https://gcp-1.seismictest.net/rpc` | Public testnet           |
| Sanvil          | `sanvil`             | 31337    | `http://127.0.0.1:8545`             | Local Seismic Anvil      |
| Local Devnet    | `localSeismicDevnet` | 5124     | `http://127.0.0.1:8545`             | Local seismic-reth --dev |

### Seismic Testnet

The public testnet for development and testing against a live Seismic network:

```typescript
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { createShieldedWalletClient, seismicTestnet } from "seismic-viem";

const client = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});
```

### Sanvil

Local development chain using [Sanvil](../../tools/sanvil.md) (Seismic's fork of Anvil). Chain ID `31337` matches Anvil/Hardhat defaults:

```typescript
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { createShieldedWalletClient, sanvil } from "seismic-viem";

const client = await createShieldedWalletClient({
  chain: sanvil,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});
```

### Local Devnet

For running a local `seismic-reth --dev` node. Uses chain ID `5124` (same as testnet) but connects to `localhost`:

```typescript
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { createShieldedWalletClient, localSeismicDevnet } from "seismic-viem";

const client = await createShieldedWalletClient({
  chain: localSeismicDevnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});
```

## Choosing a Chain

```
Are you developing against a live network?
  -> Use seismicTestnet

Are you running Sanvil locally for rapid iteration?
  -> Use sanvil

Are you running seismic-reth --dev locally?
  -> Use localSeismicDevnet

Do you need a custom chain configuration?
  -> Use createSeismicDevnet()
```

## SEISMIC_TX_TYPE

All chain configs use the Seismic transaction type constant:

```typescript
import { SEISMIC_TX_TYPE } from "seismic-viem";

console.log(SEISMIC_TX_TYPE); // 74 (0x4A)
```

The value `74` (`0x4A`) is the EIP-2718 transaction type envelope identifier for Seismic transactions. This constant is used internally by the chain formatters and encryption pipeline to identify and construct Seismic-specific transaction payloads.

## Chain Formatters

Each chain config includes `seismicChainFormatters` -- custom viem chain formatters that handle Seismic transaction fields. These formatters are applied automatically when you use a Seismic chain definition.

The formatters extend viem's standard transaction formatting to support the additional fields required by Seismic's type 0x4A transactions, including encryption metadata and signed read parameters.

{% hint style="info" %}
You do not need to configure or interact with `seismicChainFormatters` directly. They are embedded in every pre-configured chain object and in chains created via `createSeismicDevnet`.
{% endhint %}

## Custom Chain Factory

### createSeismicDevnet

Create a custom chain definition for any Seismic-compatible node:

```typescript
import { createSeismicDevnet } from "seismic-viem";

const myChain = createSeismicDevnet({
  nodeHost: "https://my-seismic-node.example.com",
  explorerUrl: "https://explorer.example.com",
});
```

#### Parameters

| Parameter     | Type     | Required | Description                                   |
| ------------- | -------- | -------- | --------------------------------------------- |
| `nodeHost`    | `string` | Yes      | Base URL of the Seismic node (without `/rpc`) |
| `explorerUrl` | `string` | No       | Block explorer URL for the chain              |

The factory returns a viem `Chain` object with:

- Chain ID `5124`
- RPC URL set to `{nodeHost}/rpc`
- Seismic chain formatters included
- Optional block explorer configuration

### Helper Factories

Convenience factories for common Seismic infrastructure:

```typescript
import { createSeismicAzTestnet, createSeismicGcpTestnet } from "seismic-viem";

// Azure-hosted testnet instance N
const azChain = createSeismicAzTestnet(1);

// GCP-hosted testnet instance N
const gcpChain = createSeismicGcpTestnet(1);
```

These generate chain configs pointing to numbered Seismic testnet instances on Azure and GCP infrastructure respectively.

## SeismicTransactionRequest

Seismic chain configs use a custom transaction request type that extends viem's standard `TransactionRequest` with Seismic-specific fields:

```typescript
interface SeismicTxExtras {
  encryptionPubkey: Hex; // TEE's ECDH public key
  encryptionNonce: Hex; // AES-GCM nonce for calldata encryption
  messageVersion: number; // Seismic message format version
  recentBlockHash: Hex; // Recent block hash for replay protection
  expiresAtBlock: bigint; // Block number at which the transaction expires
  signedRead: boolean; // Whether this is a signed read request
}
```

{% hint style="info" %}
You do not need to populate these fields manually. The shielded wallet client's encryption pipeline fills them automatically when sending transactions. They are documented here for reference and debugging purposes.
{% endhint %}

| Field              | Type      | Description                                               |
| ------------------ | --------- | --------------------------------------------------------- |
| `encryptionPubkey` | `Hex`     | The TEE's ECDH public key, fetched during client creation |
| `encryptionNonce`  | `Hex`     | Random nonce for AES-GCM encryption of calldata           |
| `messageVersion`   | `number`  | Version of the Seismic message format                     |
| `recentBlockHash`  | `Hex`     | Recent block hash used for transaction replay protection  |
| `expiresAtBlock`   | `bigint`  | Block number after which the transaction becomes invalid  |
| `signedRead`       | `boolean` | `true` for signed read requests, `false` for standard txs |

## See Also

- [Installation](installation.md) -- Install seismic-viem and viem
- [Seismic Viem Overview](./) -- Full SDK overview and architecture
- Shielded Wallet Client -- Create a client using a chain config
- Encryption -- How calldata encryption uses chain-level formatters
