---
description: Install seismic-react and configure peer dependencies
icon: download
---

# Installation

## Prerequisites

| Requirement            | Version | Notes                   |
| ---------------------- | ------- | ----------------------- |
| Node.js                | 18+     | LTS recommended         |
| React                  | ^18     | Peer dependency         |
| wagmi                  | ^2.0.0  | Peer dependency         |
| viem                   | 2.x     | Peer dependency         |
| seismic-viem           | >=1.1.1 | Seismic transport layer |
| @rainbow-me/rainbowkit | ^2.0.0  | Optional, for wallet UI |

## Install

```bash
npm install seismic-react
```

```bash
yarn add seismic-react
```

```bash
pnpm add seismic-react
```

```bash
bun add seismic-react
```

## Peer Dependencies

`seismic-react` requires several peer dependencies that your project must provide:

| Package                 | Purpose                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------- |
| `react`                 | React runtime                                                                       |
| `wagmi`                 | Ethereum React hooks and wallet connectors                                          |
| `viem`                  | TypeScript Ethereum library (used internally by wagmi)                              |
| `seismic-viem`          | Seismic transport layer providing `ShieldedPublicClient` and `ShieldedWalletClient` |
| `@tanstack/react-query` | Async state management (required by wagmi)                                          |

Install all required peer dependencies:

```bash
npm install react wagmi viem seismic-viem @tanstack/react-query
```

{% hint style="info" %}
If you are using RainbowKit for wallet UI, also install `@rainbow-me/rainbowkit`:

```bash
npm install @rainbow-me/rainbowkit
```

{% endhint %}

## Wagmi Config Setup

Create a wagmi config with Seismic chain definitions:

```typescript
import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { seismicTestnet } from "seismic-react/rainbowkit";

const config = getDefaultConfig({
  appName: "My Seismic App",
  projectId: "YOUR_WALLETCONNECT_PROJECT_ID",
  chains: [seismicTestnet],
});
```

{% hint style="warning" %}
Replace `YOUR_WALLETCONNECT_PROJECT_ID` with a project ID from [WalletConnect Cloud](https://cloud.walletconnect.com/).
{% endhint %}

## Minimal Working App

```typescript
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RainbowKitProvider, getDefaultConfig } from '@rainbow-me/rainbowkit'
import { ShieldedWalletProvider } from 'seismic-react'
import { seismicTestnet } from 'seismic-react/rainbowkit'

const config = getDefaultConfig({
  appName: 'My Seismic App',
  projectId: 'YOUR_WALLETCONNECT_PROJECT_ID',
  chains: [seismicTestnet],
})

const queryClient = new QueryClient()

function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider>
          <ShieldedWalletProvider config={config}>
            <YourApp />
          </ShieldedWalletProvider>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}
```

## Package Exports

`seismic-react` provides two entry points:

| Entry Point                | Contents                                                                                                                                |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `seismic-react`            | Main entry -- `ShieldedWalletProvider`, `useShieldedWallet`, `useShieldedContract`, `useShieldedWriteContract`, `useSignedReadContract` |
| `seismic-react/rainbowkit` | Chain configs -- `seismicTestnet`, `sanvil`, `localSeismicDevnet`, `createSeismicDevnet`                                                |

```typescript
// Main entry
import {
  ShieldedWalletProvider,
  useShieldedWallet,
  useShieldedContract,
  useShieldedWriteContract,
  useSignedReadContract,
} from "seismic-react";

// Chain configs for RainbowKit
import { seismicTestnet, sanvil } from "seismic-react/rainbowkit";
```

## TypeScript

TypeScript >= 5.0.4 is an optional peer dependency. `seismic-react` ships with full type definitions and provides type inference from contract ABIs when using hooks like `useShieldedWriteContract` and `useSignedReadContract`.

No additional `@types/*` packages are needed.

## Troubleshooting

### Module Not Found: seismic-viem

Ensure `seismic-viem` is installed as a peer dependency:

```bash
npm install seismic-viem
```

### wagmi Version Mismatch

`seismic-react` requires wagmi v2. If you have wagmi v1 installed, upgrade:

```bash
npm install wagmi@latest viem@latest
```

### RainbowKit Chain Not Appearing

Make sure you import chain configs from the `seismic-react/rainbowkit` entry point, not from the main entry:

```typescript
// Correct
import { seismicTestnet } from "seismic-react/rainbowkit";

// Incorrect -- will not resolve
import { seismicTestnet } from "seismic-react";
```

### React Version Conflicts

If you encounter React version conflicts in a monorepo, ensure all packages resolve to the same React version. Add a `resolutions` field (Yarn) or `overrides` field (npm) to your root `package.json`:

```json
{
  "resolutions": {
    "react": "^18.0.0",
    "react-dom": "^18.0.0"
  }
}
```

## See Also

- [ShieldedWalletProvider](shielded-wallet-provider.md) -- Provider setup and configuration
- [Hooks Overview](hooks/) -- Available React hooks
- [RainbowKit Guide](wallet-guides/rainbowkit.md) -- Wallet UI integration
- [Privy Guide](wallet-guides/privy.md) -- Embedded wallet setup
