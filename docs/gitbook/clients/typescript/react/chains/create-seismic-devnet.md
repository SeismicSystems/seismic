---
description: Factory function for custom Seismic chain configurations
icon: gear
---

# createSeismicDevnet

Factory function that creates a RainbowKit-compatible chain configuration for any Seismic node. Use this when the pre-configured chains (`seismicTestnet`, `sanvil`, `localSeismicDevnet`) do not match your node's host.

## Import

```typescript
import { createSeismicDevnet } from "seismic-react/rainbowkit";
```

## Parameters

| Parameter     | Type     | Required | Description                                          |
| ------------- | -------- | -------- | ---------------------------------------------------- |
| `nodeHost`    | `string` | Yes      | Hostname for the node (e.g. `gcp-1.seismictest.net`) |
| `explorerUrl` | `string` | No       | Block explorer URL                                   |

## Return Type

`RainbowKitChain` -- a chain object compatible with RainbowKit's `getDefaultConfig` and wagmi's `createConfig`.

The returned chain has:

- **Chain ID**: `5124`
- **Name**: `Seismic`
- **Native Currency**: ETH (18 decimals)
- **RPC (HTTPS)**: `https://<nodeHost>/rpc`
- **RPC (WSS)**: `wss://<nodeHost>/ws`
- **Seismic transaction formatters**

## Usage

### Basic

```typescript
import { createSeismicDevnet } from "seismic-react/rainbowkit";

const myDevnet = createSeismicDevnet({
  nodeHost: "my-node.example.com",
});
// RPC: https://my-node.example.com/rpc
// WSS: wss://my-node.example.com/ws
```

### With RainbowKit

```typescript
import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { createSeismicDevnet } from "seismic-react/rainbowkit";

const myChain = createSeismicDevnet({
  nodeHost: "my-node.example.com",
});

const config = getDefaultConfig({
  appName: "My App",
  projectId: "YOUR_PROJECT_ID",
  chains: [myChain],
});
```

### With wagmi Config

```typescript
import { http, createConfig } from "wagmi";
import { createSeismicDevnet } from "seismic-react/rainbowkit";

const myChain = createSeismicDevnet({
  nodeHost: "my-node.example.com",
});

const config = createConfig({
  chains: [myChain],
  transports: {
    [myChain.id]: http(),
  },
});
```

### With Explorer URL

```typescript
const myChain = createSeismicDevnet({
  nodeHost: "my-node.example.com",
  explorerUrl: "https://explorer.example.com",
});
```

## Notes

- The `nodeHost` parameter should be the bare hostname without a protocol prefix or path. HTTPS and WSS URLs are constructed automatically.
- The Seismic icon is included automatically for display in RainbowKit's chain selector.
- The underlying implementation delegates to `createSeismicDevnet` from `seismic-viem` and wraps the result with RainbowKit metadata.

## See Also

- [Chains Overview](./) - All supported chains
- [Seismic Testnet](seismic-testnet.md) - Public testnet configuration
- [Sanvil](sanvil.md) - Local development chains
- [Wallet Guides](../wallet-guides/) - RainbowKit setup guides
