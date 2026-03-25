---
icon: gear
---

# Ch 1: Project Setup and Providers

In this chapter, you'll set up the React frontend project and configure the provider stack that enables shielded wallet interactions in the browser. _Estimated time: ~15 minutes_

### Creating the web package

From the root of your `clown-beatdown` monorepo, create a new Vite + React + TypeScript project:

```bash
cd packages
bun create vite web --template react-ts
cd web
```

### Install dependencies

Install the core dependencies for wallet connection, shielded interactions, UI, and animations:

```bash
bun add seismic-react@1.1.1 seismic-viem@1.1.1 viem@^2.22.3 \
  wagmi@^2.0.0 @rainbow-me/rainbowkit@^2.0.0 \
  @tanstack/react-query@^5.55.3 \
  @mui/material@^6.4.3 @emotion/react @emotion/styled \
  framer-motion@^12.7.3 react-router-dom@^7.1.4 \
  react-toastify@^11.0.5 use-sound@^5.0.0
```

### Configure Vite

Update `vite.config.ts` to add a path alias for cleaner imports:

```typescript
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
```

### Environment variables

Create a `.env` file in `packages/web`:

```properties
VITE_CHAIN_ID=31337
VITE_RPC_URL=http://127.0.0.1:8545
VITE_FAUCET_URL=https://faucet-2.seismicdev.net/
```

`VITE_CHAIN_ID` determines which chain the app connects to — `31337` is the local `sanvil` node.

### Setting up the provider stack

The key architectural pattern in a seismic-react app is the **provider stack**. This wraps your application in the context providers needed for wallet connection and shielded operations.

Create `src/App.tsx`:

```typescript
import {
  RainbowKitProvider,
  darkTheme,
  getDefaultConfig,
} from '@rainbow-me/rainbowkit'
import '@rainbow-me/rainbowkit/styles.css'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider } from 'seismic-react'
import { sanvil, seismicTestnet } from 'seismic-viem'
import { http, WagmiProvider, useAccount } from 'wagmi'

import { AuthProvider } from './components/chain/WalletConnectButton'
import Home from './pages/Home'

// Select chain based on environment variable
const chainConfig =
  import.meta.env.VITE_CHAIN_ID === String(sanvil.id) ? sanvil : seismicTestnet

// Configure wagmi with RainbowKit defaults
const config = getDefaultConfig({
  appName: 'Clown Beatdown',
  projectId: 'YOUR_WALLETCONNECT_PROJECT_ID',
  chains: [chainConfig],
  transports: {
    [chainConfig.id]: http(import.meta.env.VITE_RPC_URL),
  },
})

const queryClient = new QueryClient()

function Providers({ children }: { children: React.ReactNode }) {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider theme={darkTheme()}>
          <ShieldedWalletProvider>
            <AuthProvider>{children}</AuthProvider>
          </ShieldedWalletProvider>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}

export default function App() {
  return (
    <Providers>
      <Home />
    </Providers>
  )
}
```

### What's happening here?

The provider stack nests four layers, each adding functionality:

1. **WagmiProvider** — manages wallet connections and chain state via wagmi hooks (`useAccount`, `useConnect`, etc.)
2. **QueryClientProvider** — provides React Query for caching and background data fetching
3. **RainbowKitProvider** — adds a polished wallet connect modal UI
4. **ShieldedWalletProvider** — the Seismic-specific layer from `seismic-react` that derives a shielded wallet client from the connected wagmi account, enabling shielded reads and writes

This is the same pattern you'd use for any Seismic dApp. The `ShieldedWalletProvider` is the only Seismic-specific addition to a standard wagmi + RainbowKit setup.
