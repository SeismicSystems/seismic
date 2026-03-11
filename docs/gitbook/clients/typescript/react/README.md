---
description: >-
  React hooks and providers for Seismic, composing with wagmi to add shielded
  wallet management, encrypted transactions, and signed reads to React apps.
icon: atom
---

# Seismic React

React SDK (v1.1.1) for [Seismic](https://seismic.systems), built on [wagmi](https://wagmi.sh/) 2.0+ and [viem](https://viem.sh/) 2.x. Provides `ShieldedWalletProvider` context and hooks for encrypted transactions and signed reads in React applications.

```bash
npm install seismic-react
```

## Quick Start

Minimal setup with RainbowKit:

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

## Architecture

The SDK wraps wagmi's connector layer to inject Seismic's shielded wallet and public clients:

```
wagmi config
  └─ ShieldedWalletProvider
       ├─ ShieldedPublicClient  (encrypted reads)
       ├─ ShieldedWalletClient  (encrypted writes)
       └─ Hooks
            ├─ useShieldedWallet          Access wallet/public clients
            ├─ useShieldedContract         Contract instance with ABI binding
            ├─ useShieldedWriteContract    Encrypted write transactions
            └─ useSignedReadContract       Signed, encrypted reads
```

## Documentation Navigation

### Getting Started

| Section                                                   | Description                                         |
| --------------------------------------------------------- | --------------------------------------------------- |
| **[Installation](installation.md)**                       | Package setup, peer dependencies, and configuration |
| **[ShieldedWalletProvider](shielded-wallet-provider.md)** | React context provider for shielded clients         |

### Hooks Reference

| Section                                                   | Description                                 |
| --------------------------------------------------------- | ------------------------------------------- |
| **[Hooks Overview](hooks/)**                              | Summary of all available hooks              |
| **[useShieldedWallet](hooks/useshieldedwallet.md)**       | Access shielded wallet and public clients   |
| **[useShieldedContract](hooks/useshieldedcontract.md)**   | Create a contract instance with ABI binding |
| **[useShieldedWriteContract](hooks/useshieldedwrite.md)** | Send encrypted write transactions           |
| **[useSignedReadContract](hooks/useshieldedread.md)**     | Perform signed, encrypted read calls        |

### Wallet Guides

| Section                                       | Description                           |
| --------------------------------------------- | ------------------------------------- |
| **[Wallet Guides Overview](wallet-guides/)**  | Connecting different wallet providers |
| **[RainbowKit](wallet-guides/rainbowkit.md)** | Setup with RainbowKit wallet UI       |
| **[Privy](wallet-guides/privy.md)**           | Embedded wallets with Privy           |
| **[AppKit](wallet-guides/appkit.md)**         | WalletConnect AppKit integration      |

## Quick Links

### By Task

- **Connect a shielded wallet** -> [ShieldedWalletProvider](shielded-wallet-provider.md)
- **Read shielded state** -> [useSignedReadContract](hooks/useshieldedread.md)
- **Send an encrypted transaction** -> [useShieldedWriteContract](hooks/useshieldedwrite.md)
- **Get a contract instance** -> [useShieldedContract](hooks/useshieldedcontract.md)
- **Install the package** -> [Installation](installation.md)
- **Set up RainbowKit** -> [RainbowKit Guide](wallet-guides/rainbowkit.md)

### By Component

- **Provider** -> [ShieldedWalletProvider](shielded-wallet-provider.md)
- **Hooks** -> [useShieldedWallet](hooks/useshieldedwallet.md), [useShieldedContract](hooks/useshieldedcontract.md), [useShieldedWriteContract](hooks/useshieldedwrite.md), [useSignedReadContract](hooks/useshieldedread.md)
- **Chain configs** -> `seismicTestnet`, `sanvil`, `localSeismicDevnet`, `createSeismicDevnet`

## Features

- **Shielded Transactions** -- Encrypt calldata before sending via `useShieldedWriteContract`
- **Signed Reads** -- Prove caller identity in `eth_call` with `useSignedReadContract`
- **Contract Abstraction** -- ABI-bound contract instances via `useShieldedContract`
- **wagmi/RainbowKit Integration** -- Drop-in provider that composes with the standard wagmi stack
- **TypeScript Support** -- Full type inference from contract ABIs

## Next Steps

1. **[Install seismic-react](installation.md)** -- Add the package and peer dependencies
2. **[Set up ShieldedWalletProvider](shielded-wallet-provider.md)** -- Wrap your app with the provider
3. **[Connect a wallet](wallet-guides/rainbowkit.md)** -- Choose a wallet integration
4. **[Use hooks](hooks/)** -- Read and write shielded contract state
