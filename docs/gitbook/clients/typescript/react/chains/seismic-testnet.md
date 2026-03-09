---
description: Seismic public testnet chain configuration
icon: cloud
---

# Seismic Testnet

The Seismic public testnet is the primary network for development and testing. `seismicTestnet` is a RainbowKit-compatible chain object that wraps the `seismic-viem` testnet definition with RainbowKit metadata.

## Configuration

| Property        | Value                                   |
| --------------- | --------------------------------------- |
| Chain ID        | `5124`                                  |
| Name            | `Seismic`                               |
| RPC (HTTPS)     | `https://gcp-1.seismictest.net/rpc`     |
| RPC (WSS)       | `wss://gcp-1.seismictest.net/ws`        |
| Explorer        | `https://seismic-testnet.socialscan.io` |
| Native Currency | ETH (18 decimals)                       |

## Import

```typescript
import { seismicTestnet } from "seismic-react/rainbowkit";
```

## Usage

### With RainbowKit

```typescript
import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { seismicTestnet } from "seismic-react/rainbowkit";

const config = getDefaultConfig({
  appName: "My App",
  projectId: "YOUR_PROJECT_ID",
  chains: [seismicTestnet],
});
```

### With wagmi Config

```typescript
import { http, createConfig } from "wagmi";
import { seismicTestnet } from "seismic-react/rainbowkit";

const config = createConfig({
  chains: [seismicTestnet],
  transports: {
    [seismicTestnet.id]: http(),
  },
});
```

## Notes

- Chain ID `5124` is used for EIP-155 replay protection and EIP-712 typed data signing
- The testnet supports all Seismic protocol features including shielded transactions and signed reads
- The Seismic icon is included automatically for display in RainbowKit's chain selector

## See Also

- [Chains Overview](./) - All supported chains
- [Sanvil](sanvil.md) - Local development chains
- [createSeismicDevnet](create-seismic-devnet.md) - Custom chain factory
- [Wallet Guides](../wallet-guides/) - RainbowKit setup guides
