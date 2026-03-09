---
description: React context provider for shielded wallet and public client
icon: wallet
---

# ShieldedWalletProvider

React context component that wraps wagmi's connector client to provide `ShieldedPublicClient` and `ShieldedWalletClient` from [seismic-viem](https://github.com/SeismicSystems/seismic-viem). This is the core provider that all Seismic React hooks depend on.

## Import

```typescript
import { ShieldedWalletProvider } from "seismic-react";
```

## Props

| Prop                      | Type                                               | Required | Description                                              |
| ------------------------- | -------------------------------------------------- | -------- | -------------------------------------------------------- |
| `children`                | `React.ReactNode`                                  | Yes      | Child components that can access shielded wallet context |
| `config`                  | `Config` (from wagmi)                              | Yes      | wagmi configuration object                               |
| `options`                 | `object`                                           | No       | Additional configuration options                         |
| `options.publicTransport` | `Transport`                                        | No       | Custom transport for the public client                   |
| `options.publicChain`     | `Chain`                                            | No       | Custom chain for the public client                       |
| `options.onAddressChange` | `(params: OnAddressChangeParams) => Promise<void>` | No       | Callback fired when the connected wallet address changes |

### OnAddressChangeParams

```typescript
type OnAddressChangeParams = {
  publicClient: ShieldedPublicClient;
  walletClient: ShieldedWalletClient;
  address: Hex;
};
```

## Context Value

The provider exposes a `WalletClientContextType` to all descendant components via React context:

```typescript
interface WalletClientContextType {
  publicClient: ShieldedPublicClient | null;
  walletClient: ShieldedWalletClient | null;
  address: Hex | null;
  error: string | null;
  loaded: boolean;
}
```

| Field          | Description                                                                                           |
| -------------- | ----------------------------------------------------------------------------------------------------- |
| `publicClient` | Shielded public client for encrypted reads. `null` until initialization completes.                    |
| `walletClient` | Shielded wallet client for encrypted writes. `null` until a wallet is connected.                      |
| `address`      | Connected wallet address. `null` when no wallet is connected.                                         |
| `error`        | Error message if client creation fails. `null` when healthy.                                          |
| `loaded`       | `true` once the provider has finished its initial setup, regardless of whether a wallet is connected. |

{% hint style="info" %}
Access these values through the [`useShieldedWallet`](hooks/useshieldedwallet.md) hook rather than consuming the context directly.
{% endhint %}

## Setup

### Basic Setup with RainbowKit

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

export default function App({ children }: { children: React.ReactNode }) {
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

### Next.js Setup

For Next.js, create the provider in a client component:

```typescript
'use client'

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

Then wrap your layout:

```typescript
// app/layout.tsx
import { Providers } from './providers'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html>
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  )
}
```

## Provider Nesting Order

The providers must be nested in this order:

```
WagmiProvider
  └─ QueryClientProvider
       └─ RainbowKitProvider
            └─ ShieldedWalletProvider
                 └─ Your App
```

{% hint style="warning" %}
`ShieldedWalletProvider` must be inside `WagmiProvider` because it reads the connected wallet from wagmi's context. Placing it outside will cause a runtime error.
{% endhint %}

## Lifecycle

The provider follows this initialization sequence:

1. **Public client creation** -- On mount, creates a `ShieldedPublicClient` using the chain and transport from the wagmi config (or from `options.publicChain` / `options.publicTransport` if provided). This client is available even when no wallet is connected.

2. **Wallet client creation** -- When a wallet connects through wagmi, the provider creates a `ShieldedWalletClient` from the connector's client. This enables encrypted write transactions.

3. **`onAddressChange` callback** -- If provided, called after both clients are ready with the new address. Use this to load user-specific data or initialize contract state.

4. **Reconnection handling** -- When the user switches accounts or reconnects, the provider recreates the wallet client and fires `onAddressChange` again.

## Usage with onAddressChange

The `onAddressChange` callback is useful for initializing state when a wallet connects:

```typescript
import { ShieldedWalletProvider } from 'seismic-react'
import type { OnAddressChangeParams } from 'seismic-react'

function App() {
  const handleAddressChange = async ({
    publicClient,
    walletClient,
    address,
  }: OnAddressChangeParams) => {
    console.log('Connected:', address)

    // Example: read user's shielded balance on connect
    const balance = await publicClient.readContract({
      address: TOKEN_ADDRESS,
      abi: tokenAbi,
      functionName: 'balanceOf',
      args: [address],
    })
    console.log('Balance:', balance)
  }

  return (
    <ShieldedWalletProvider
      config={config}
      options={{ onAddressChange: handleAddressChange }}
    >
      <YourApp />
    </ShieldedWalletProvider>
  )
}
```

## Error Handling

The provider captures errors during client creation and exposes them through the context:

| Error State                  | Cause                                         | Resolution                                     |
| ---------------------------- | --------------------------------------------- | ---------------------------------------------- |
| Public client creation fails | Invalid chain config or transport             | Check wagmi config and chain definitions       |
| Wallet client creation fails | Connector incompatibility or network mismatch | Ensure the wallet supports the target chain    |
| `onAddressChange` throws     | Error in your callback                        | The error is caught and set on `context.error` |

Access the error state through `useShieldedWallet`:

```typescript
import { useShieldedWallet } from 'seismic-react'

function MyComponent() {
  const { error, loaded } = useShieldedWallet()

  if (!loaded) return <div>Loading...</div>
  if (error) return <div>Error: {error}</div>

  return <div>Connected</div>
}
```

## See Also

- [Installation](installation.md) -- Package setup and peer dependencies
- [useShieldedWallet](hooks/useshieldedwallet.md) -- Hook to consume the provider context
- [Hooks Overview](hooks/) -- All available hooks
- [RainbowKit Guide](wallet-guides/rainbowkit.md) -- Wallet UI integration
- [Privy Guide](wallet-guides/privy.md) -- Embedded wallet setup
