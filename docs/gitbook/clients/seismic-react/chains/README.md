---
description: RainbowKit-compatible chain configurations for Seismic networks
icon: link
---

# Chains

`seismic-react` provides pre-configured chain objects compatible with RainbowKit's `getDefaultConfig`. Each chain wraps a [`seismic-viem`](../../seismic-viem/) chain definition with RainbowKit metadata (icon, etc.).

## Import

```typescript
import {
  seismicTestnet,
  sanvil,
  localSeismicDevnet,
  createSeismicDevnet,
} from "seismic-react/rainbowkit";
```

## Available Chains

| Chain                                     | Export                  | Chain ID | Description                      |
| ----------------------------------------- | ----------------------- | -------- | -------------------------------- |
| [Seismic Testnet](seismic-testnet.md)     | `seismicTestnet`        | `5124`   | Public testnet                   |
| [Sanvil](sanvil.md)                       | `sanvil`                | `31337`  | Local Sanvil dev node            |
| [Local Devnet](sanvil.md)                 | `localSeismicDevnet`    | --       | Local seismic-reth `--dev`       |
| [Custom Devnet](create-seismic-devnet.md) | `createSeismicDevnet()` | `5124`   | Factory for custom chain configs |

## Usage with RainbowKit

```typescript
import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { seismicTestnet } from "seismic-react/rainbowkit";

const config = getDefaultConfig({
  appName: "My App",
  projectId: "YOUR_PROJECT_ID",
  chains: [seismicTestnet],
});
```

## Choosing a Chain

```
Are you deploying to the public testnet?
  -> Use seismicTestnet

Are you developing locally with Sanvil?
  -> Use sanvil

Are you running a local seismic-reth node in --dev mode?
  -> Use localSeismicDevnet

Are you connecting to a custom or self-hosted Seismic node?
  -> Use createSeismicDevnet()
```

## Relationship to seismic-viem Chains

Each chain export is a thin wrapper around the corresponding `seismic-viem` chain object. The wrapper adds RainbowKit-specific metadata (such as `iconUrl`) while preserving all underlying chain properties -- chain ID, RPC URLs, native currency, block explorers, and transaction formatters.

{% hint style="info" %}
If you are not using RainbowKit, you can use the `seismic-viem` chain objects directly with wagmi.
{% endhint %}

## Pages

| Page                                            | Description                                      |
| ----------------------------------------------- | ------------------------------------------------ |
| [Seismic Testnet](seismic-testnet.md)           | Public testnet configuration and usage           |
| [Sanvil](sanvil.md)                             | Local development chains (Sanvil + seismic-reth) |
| [createSeismicDevnet](create-seismic-devnet.md) | Factory for custom chain configurations          |

## See Also

- [ShieldedWalletProvider](../shielded-wallet-provider.md) - Provider that accepts chain config
- [Wallet Guides](../wallet-guides/) - RainbowKit and wallet setup guides
- [Installation](../installation.md) - Package setup
