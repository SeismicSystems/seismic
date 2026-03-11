---
description: Set up Privy with Seismic for email and social login
icon: key
---

# Privy

Privy provides email, social, and embedded wallet authentication. This guide shows how to integrate Privy with Seismic React for apps that need flexible onboarding beyond browser extension wallets.

## Prerequisites

```bash
npm install @privy-io/react-auth @privy-io/wagmi wagmi viem @tanstack/react-query seismic-react seismic-viem
```

{% hint style="info" %}
You need a Privy App ID from the [Privy Dashboard](https://dashboard.privy.io/).
{% endhint %}

## Step 1: Configure Privy with Seismic Chain

Define the Seismic chain configuration for Privy:

```typescript
import { seismicTestnet } from "seismic-react/rainbowkit";

// Privy needs the chain in viem format
const seismicChain = {
  id: seismicTestnet.id,
  name: seismicTestnet.name,
  nativeCurrency: seismicTestnet.nativeCurrency,
  rpcUrls: seismicTestnet.rpcUrls,
  blockExplorers: seismicTestnet.blockExplorers,
};
```

## Step 2: Set Up wagmi Config

Create a wagmi config using Privy's wagmi integration:

```typescript
import { createConfig } from "@privy-io/wagmi";
import { http } from "viem";
import { seismicTestnet } from "seismic-react/rainbowkit";

const wagmiConfig = createConfig({
  chains: [seismicTestnet],
  transports: {
    [seismicTestnet.id]: http(),
  },
});
```

## Step 3: Set Up Providers

Nest the providers in the correct order -- Privy wraps wagmi, and `ShieldedWalletProvider` goes inside:

```typescript
import { PrivyProvider } from '@privy-io/react-auth'
import { WagmiProvider } from '@privy-io/wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider } from 'seismic-react'

const queryClient = new QueryClient()

function App({ children }: { children: React.ReactNode }) {
  return (
    <PrivyProvider
      appId="YOUR_PRIVY_APP_ID"
      config={{
        defaultChain: seismicChain,
        supportedChains: [seismicChain],
        embeddedWallets: {
          createOnLogin: 'users-without-wallets',
        },
      }}
    >
      <QueryClientProvider client={queryClient}>
        <WagmiProvider config={wagmiConfig}>
          <ShieldedWalletProvider config={wagmiConfig}>
            {children}
          </ShieldedWalletProvider>
        </WagmiProvider>
      </QueryClientProvider>
    </PrivyProvider>
  )
}
```

{% hint style="info" %}
Privy's embedded wallets work seamlessly with Seismic -- users don't need a browser extension. Privy creates a wallet automatically when users sign in with email or social accounts.
{% endhint %}

## Step 4: Add Login Button

Use Privy's hooks to trigger the login modal:

```typescript
import { usePrivy } from '@privy-io/react-auth'

function LoginButton() {
  const { login, logout, authenticated, user } = usePrivy()

  if (authenticated) {
    return (
      <div>
        <p>Logged in as {user?.email?.address || user?.wallet?.address}</p>
        <button onClick={logout}>Log out</button>
      </div>
    )
  }

  return <button onClick={login}>Sign in</button>
}
```

## Step 5: Use Shielded Hooks

Once authenticated, use `seismic-react` hooks as normal:

```typescript
import { useShieldedWallet } from 'seismic-react'

function MyComponent() {
  const { walletClient, loaded, error } = useShieldedWallet()

  if (!loaded) return <div>Initializing shielded wallet...</div>
  if (error) return <div>Error: {error}</div>
  if (!walletClient) return <div>Sign in to get started.</div>

  return <div>Shielded wallet ready!</div>
}
```

## Complete Example

```typescript
'use client'

import { PrivyProvider, usePrivy } from '@privy-io/react-auth'
import { WagmiProvider, createConfig } from '@privy-io/wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider, useShieldedWallet } from 'seismic-react'
import { seismicTestnet } from 'seismic-react/rainbowkit'
import { http } from 'viem'

const seismicChain = {
  id: seismicTestnet.id,
  name: seismicTestnet.name,
  nativeCurrency: seismicTestnet.nativeCurrency,
  rpcUrls: seismicTestnet.rpcUrls,
  blockExplorers: seismicTestnet.blockExplorers,
}

const wagmiConfig = createConfig({
  chains: [seismicTestnet],
  transports: {
    [seismicTestnet.id]: http(),
  },
})

const queryClient = new QueryClient()

function LoginButton() {
  const { login, logout, authenticated, user } = usePrivy()

  if (authenticated) {
    return (
      <div>
        <p>Logged in as {user?.email?.address || user?.wallet?.address}</p>
        <button onClick={logout}>Log out</button>
      </div>
    )
  }

  return <button onClick={login}>Sign in</button>
}

function WalletStatus() {
  const { walletClient, publicClient, loaded, error } = useShieldedWallet()

  if (!loaded) return <p>Initializing shielded wallet...</p>
  if (error) return <p>Error: {error}</p>
  if (!walletClient) return <p>Sign in to get started.</p>

  return (
    <div>
      <p>Shielded wallet ready</p>
      <p>Public client: {publicClient ? 'Available' : 'Loading...'}</p>
    </div>
  )
}

export default function App() {
  return (
    <PrivyProvider
      appId="YOUR_PRIVY_APP_ID"
      config={{
        defaultChain: seismicChain,
        supportedChains: [seismicChain],
        embeddedWallets: {
          createOnLogin: 'users-without-wallets',
        },
      }}
    >
      <QueryClientProvider client={queryClient}>
        <WagmiProvider config={wagmiConfig}>
          <ShieldedWalletProvider config={wagmiConfig}>
            <LoginButton />
            <WalletStatus />
          </ShieldedWalletProvider>
        </WagmiProvider>
      </QueryClientProvider>
    </PrivyProvider>
  )
}
```

## See Also

- [Wallet Guides Overview](./) -- Comparison of wallet libraries
- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Provider reference and options
- [useShieldedWallet](../hooks/useshieldedwallet.md) -- Access shielded wallet context
- [RainbowKit Guide](rainbowkit.md) -- Traditional wallet modal alternative
- [AppKit Guide](appkit.md) -- WalletConnect modal alternative
