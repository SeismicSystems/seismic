---
description: Set up AppKit (WalletConnect) with Seismic
icon: grid-2
---

# AppKit

AppKit (formerly Web3Modal) by WalletConnect provides a wallet connection modal with support for 300+ wallets. This guide shows how to integrate AppKit with Seismic React.

## Prerequisites

```bash
npm install @reown/appkit @reown/appkit-adapter-wagmi wagmi viem @tanstack/react-query seismic-react seismic-viem
```

{% hint style="warning" %}
You need a WalletConnect Project ID from [WalletConnect Cloud](https://cloud.walletconnect.com/).
{% endhint %}

## Step 1: Configure wagmi with AppKit

Create a wagmi adapter and initialize AppKit with Seismic chains:

```typescript
import { createAppKit } from "@reown/appkit/react";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";
import { seismicTestnet } from "seismic-react/rainbowkit";

const projectId = "YOUR_WALLETCONNECT_PROJECT_ID";

const wagmiAdapter = new WagmiAdapter({
  projectId,
  chains: [seismicTestnet],
  networks: [seismicTestnet],
});

createAppKit({
  adapters: [wagmiAdapter],
  projectId,
  networks: [seismicTestnet],
  metadata: {
    name: "My Seismic App",
    description: "A Seismic-powered dApp",
    url: "https://myapp.com",
    icons: ["https://myapp.com/icon.png"],
  },
});
```

## Step 2: Set Up Providers

Nest the providers with `ShieldedWalletProvider` inside:

```typescript
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider } from 'seismic-react'

const queryClient = new QueryClient()

function App({ children }: { children: React.ReactNode }) {
  return (
    <WagmiProvider config={wagmiAdapter.wagmiConfig}>
      <QueryClientProvider client={queryClient}>
        <ShieldedWalletProvider config={wagmiAdapter.wagmiConfig}>
          {children}
        </ShieldedWalletProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}
```

{% hint style="info" %}
AppKit does not require a wrapper provider component like RainbowKit or Privy. The `createAppKit` call initializes it globally, so `ShieldedWalletProvider` goes directly inside `QueryClientProvider`.
{% endhint %}

## Step 3: Add the Connect Button

AppKit provides a web component for the connect button:

```typescript
function Header() {
  return (
    <header>
      <appkit-button />
    </header>
  )
}
```

{% hint style="info" %}
If you are using TypeScript, you may need to declare the web component type. Add this to a `.d.ts` file in your project:

```typescript
declare namespace JSX {
  interface IntrinsicElements {
    "appkit-button": React.DetailedHTMLProps<
      React.HTMLAttributes<HTMLElement>,
      HTMLElement
    >;
  }
}
```

{% endhint %}

## Step 4: Use Shielded Hooks

Once connected, use `seismic-react` hooks to interact with shielded contracts:

```typescript
import { useShieldedWallet } from 'seismic-react'

function MyComponent() {
  const { walletClient, loaded, error } = useShieldedWallet()

  if (!loaded) return <div>Initializing shielded wallet...</div>
  if (error) return <div>Error: {error}</div>
  if (!walletClient) return <div>Connect your wallet to get started.</div>

  return <div>Shielded wallet ready!</div>
}
```

## Complete Example

```typescript
'use client'

import { createAppKit } from '@reown/appkit/react'
import { WagmiAdapter } from '@reown/appkit-adapter-wagmi'
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider, useShieldedWallet } from 'seismic-react'
import { seismicTestnet } from 'seismic-react/rainbowkit'

const projectId = 'YOUR_WALLETCONNECT_PROJECT_ID'

const wagmiAdapter = new WagmiAdapter({
  projectId,
  chains: [seismicTestnet],
  networks: [seismicTestnet],
})

createAppKit({
  adapters: [wagmiAdapter],
  projectId,
  networks: [seismicTestnet],
  metadata: {
    name: 'My Seismic App',
    description: 'A Seismic-powered dApp',
    url: 'https://myapp.com',
    icons: ['https://myapp.com/icon.png'],
  },
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
    <WagmiProvider config={wagmiAdapter.wagmiConfig}>
      <QueryClientProvider client={queryClient}>
        <ShieldedWalletProvider config={wagmiAdapter.wagmiConfig}>
          <appkit-button />
          <WalletStatus />
        </ShieldedWalletProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}
```

## See Also

- [Wallet Guides Overview](./) -- Comparison of wallet libraries
- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Provider reference and options
- [useShieldedWallet](../hooks/useshieldedwallet.md) -- Access shielded wallet context
- [RainbowKit Guide](rainbowkit.md) -- Polished wallet modal alternative
- [Privy Guide](privy.md) -- Email/social login alternative
