---
description: Complete runnable examples for Seismic React
icon: code
---

# Examples

Complete working examples demonstrating common Seismic React patterns. Each example is self-contained and can be copied into a new project.

## Available Examples

| Example                     | Description                                                                        |
| --------------------------- | ---------------------------------------------------------------------------------- |
| [Basic dApp](basic-dapp.md) | Complete minimal dApp: provider setup, connect wallet, shielded write, signed read |

## Common Setup

Every example uses the same provider wrapper that combines RainbowKit, wagmi, and Seismic:

```typescript
import { RainbowKitProvider } from '@rainbow-me/rainbowkit'
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider } from 'seismic-react'
import { config } from './config'

const queryClient = new QueryClient()

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider>
          <ShieldedWalletProvider config={config}>
            {children}
          </ShieldedWalletProvider>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}
```

{% hint style="info" %}
`ShieldedWalletProvider` must be nested inside `WagmiProvider` and `QueryClientProvider`. It automatically creates shielded clients when a wallet connects.
{% endhint %}

## Prerequisites

- Node.js 18+
- A [WalletConnect](https://cloud.walletconnect.com/) project ID
- `seismic-react` and peer dependencies installed

```bash
npm install seismic-react seismic-viem wagmi viem @rainbow-me/rainbowkit @tanstack/react-query
```

## See Also

- [Hooks](../hooks/) - Hook API reference
- [Wallet Guides](../wallet-guides/) - RainbowKit, AppKit, and Privy integration
- [Installation](../installation.md) - Full dependency setup
