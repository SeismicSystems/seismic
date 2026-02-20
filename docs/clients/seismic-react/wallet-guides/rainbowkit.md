---
description: Set up RainbowKit with Seismic for wallet connection
icon: rainbow
---

# RainbowKit

RainbowKit is the recommended wallet connection library for Seismic React apps. It provides a polished connect modal, chain switching, and account management out of the box.

## Prerequisites

```bash
npm install @rainbow-me/rainbowkit wagmi viem @tanstack/react-query seismic-react seismic-viem
```

## Step 1: Configure wagmi

Create a wagmi config using RainbowKit's `getDefaultConfig` with Seismic chains:

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

## Step 2: Set Up Providers

Wrap your app with the provider stack. The nesting order matters:

```typescript
import { RainbowKitProvider } from '@rainbow-me/rainbowkit'
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider } from 'seismic-react'
import '@rainbow-me/rainbowkit/styles.css'

const queryClient = new QueryClient()

function App({ children }: { children: React.ReactNode }) {
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
`ShieldedWalletProvider` must be inside `RainbowKitProvider` so it can access the connected wallet from wagmi's context.
{% endhint %}

## Step 3: Add the Connect Button

RainbowKit provides a pre-built connect button component:

```typescript
import { ConnectButton } from '@rainbow-me/rainbowkit'

function Header() {
  return (
    <header>
      <ConnectButton />
    </header>
  )
}
```

## Step 4: Use Shielded Hooks

Once the providers are in place, use `seismic-react` hooks to interact with shielded contracts:

```typescript
import { useShieldedWallet } from 'seismic-react'

function MyComponent() {
  const { walletClient, loaded, error } = useShieldedWallet()

  if (!loaded) return <div>Initializing shielded wallet...</div>
  if (error) return <div>Error: {error}</div>

  return <div>Connected!</div>
}
```

## Next.js Setup

Next.js App Router requires client components for providers. Create a separate providers file:

```typescript
// app/providers.tsx
'use client'

import { RainbowKitProvider, getDefaultConfig } from '@rainbow-me/rainbowkit'
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider } from 'seismic-react'
import { seismicTestnet } from 'seismic-react/rainbowkit'
import '@rainbow-me/rainbowkit/styles.css'

const config = getDefaultConfig({
  appName: 'My Seismic App',
  projectId: 'YOUR_WALLETCONNECT_PROJECT_ID',
  chains: [seismicTestnet],
})

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

Then wrap your root layout:

```typescript
// app/layout.tsx
import { Providers } from './providers'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  )
}
```

{% hint style="info" %}
The `'use client'` directive is required because providers use React context, which is only available in client components.
{% endhint %}

## Local Development with Sanvil

For local development, use the `sanvil` chain (Seismic's local development node) instead of `seismicTestnet`:

```typescript
import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { seismicTestnet, sanvil } from "seismic-react/rainbowkit";

const config = getDefaultConfig({
  appName: "My Seismic App",
  projectId: "YOUR_WALLETCONNECT_PROJECT_ID",
  chains: [
    ...(process.env.NODE_ENV === "development" ? [sanvil] : []),
    seismicTestnet,
  ],
});
```

## Complete Example

A full working setup in a single file:

```typescript
'use client'

import { RainbowKitProvider, ConnectButton, getDefaultConfig } from '@rainbow-me/rainbowkit'
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider, useShieldedWallet } from 'seismic-react'
import { seismicTestnet } from 'seismic-react/rainbowkit'
import '@rainbow-me/rainbowkit/styles.css'

const config = getDefaultConfig({
  appName: 'My Seismic App',
  projectId: 'YOUR_WALLETCONNECT_PROJECT_ID',
  chains: [seismicTestnet],
})

const queryClient = new QueryClient()

function WalletStatus() {
  const { walletClient, publicClient, loaded, error } = useShieldedWallet()

  if (!loaded) return <p>Initializing shielded wallet...</p>
  if (error) return <p>Error: {error}</p>
  if (!walletClient) return <p>Connect your wallet to get started.</p>

  return (
    <div>
      <p>Shielded wallet ready</p>
      <p>Public client: {publicClient ? 'Available' : 'Loading...'}</p>
    </div>
  )
}

export default function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider>
          <ShieldedWalletProvider config={config}>
            <ConnectButton />
            <WalletStatus />
          </ShieldedWalletProvider>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}
```

## See Also

- [Wallet Guides Overview](./) -- Comparison of wallet libraries
- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Provider reference and options
- [useShieldedWallet](../hooks/useshieldedwallet.md) -- Access shielded wallet context
- [Privy Guide](privy.md) -- Email/social login alternative
- [AppKit Guide](appkit.md) -- WalletConnect modal alternative
