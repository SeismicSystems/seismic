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
  react-toastify@^11.0.5 use-sound@^5.0.0 \
  react-redux@^9.2.0 @reduxjs/toolkit@^2.5.1 \
  @tailwindcss/vite tailwindcss@^4
```

The Vite config below uses the SWC plugin for faster builds. Install it as a dev dependency:

```bash
bun add -d @vitejs/plugin-react-swc
```

### Copy public assets

Copy the `public/` folder from the [seismic-starter](https://github.com/SeismicSystems/seismic-starter) repo into `packages/web/public/`. This includes the clown sprites, button images, background, logo, and audio files used by the game UI.

### Configure Vite

Update `vite.config.ts`:

```typescript
import { resolve } from "path";
import { defineConfig } from "vite";

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react-swc";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  envDir: resolve(__dirname, "../.."),
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
});
```

The `envDir` points to the monorepo root so that `.env` files at the top level are available to the web package.

### Environment variables

Create a `.env` file at the monorepo root:

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
import React from 'react'
import { PropsWithChildren, useCallback } from 'react'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import {
  type OnAddressChangeParams,
  ShieldedWalletProvider,
} from 'seismic-react'
import { sanvil, seismicTestnet } from 'seismic-react/rainbowkit'
import { http } from 'viem'
import { type Config, WagmiProvider } from 'wagmi'

import { AuthProvider } from '@/components/chain/WalletConnectButton'
import Home from '@/pages/Home'
import NotFound from '@/pages/NotFound'
import { getDefaultConfig } from '@rainbow-me/rainbowkit'
import { RainbowKitProvider } from '@rainbow-me/rainbowkit'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import './App.css'

const configuredChainId = String(import.meta.env.VITE_CHAIN_ID ?? '')
const isSanvilConfig =
  configuredChainId === 'sanvil' || configuredChainId === String(sanvil.id)
const CHAIN = isSanvilConfig ? sanvil : seismicTestnet
const CHAINS = [CHAIN]

const config = getDefaultConfig({
  appName: 'Seismic Starter',
  projectId: 'd705c8eaf9e6f732e1ddb8350222cdac',
  // @ts-expect-error: this is fine
  chains: CHAINS,
  ssr: false,
})

const client = new QueryClient()

const Providers: React.FC<PropsWithChildren<{ config: Config }>> = ({
  config,
  children,
}) => {
  const publicChain = CHAINS[0]
  const publicTransport = http(publicChain.rpcUrls.default.http[0])
  const handleAddressChange = useCallback(
    async ({ publicClient, address }: OnAddressChangeParams) => {
      if (publicClient.chain.id !== sanvil.id) return

      const existingBalance = await publicClient.getBalance({ address })
      if (existingBalance > 0n) return

      const setBalance = publicClient.request as unknown as (args: {
        method: string
        params?: unknown[]
      }) => Promise<unknown>

      await setBalance({
        method: 'anvil_setBalance',
        params: [address, `0x${(10_000n * 10n ** 18n).toString(16)}`],
      })
    },
    []
  )

  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={client}>
        <RainbowKitProvider>
          <ShieldedWalletProvider
            config={config}
            options={{
              publicTransport,
              publicChain,
              onAddressChange: handleAddressChange,
            }}
          >
            <AuthProvider>{children}</AuthProvider>
          </ShieldedWalletProvider>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}

const App: React.FC = () => {
  return (
    <BrowserRouter>
      <Providers config={config}>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </Providers>
    </BrowserRouter>
  )
}

export default App
```

### What's happening here?

The provider stack nests four layers, each adding functionality:

1. **WagmiProvider** — manages wallet connections and chain state via wagmi hooks (`useAccount`, `useConnect`, etc.)
2. **QueryClientProvider** — provides React Query for caching and background data fetching
3. **RainbowKitProvider** — adds a polished wallet connect modal UI
4. **ShieldedWalletProvider** — the Seismic-specific layer from `seismic-react` that derives a shielded wallet client from the connected wagmi account, enabling shielded reads and writes. It takes `config` and `options` — the options include `publicTransport`, `publicChain`, and an `onAddressChange` callback.

The `onAddressChange` handler auto-funds new wallets when running on `sanvil` (local dev), so you don't need to manually send ETH to test accounts.

### Supporting files

Before wiring up the entry point, create the supporting modules that `main.tsx` and `App.tsx` will import.

**Redux store** — Create `src/store/store.ts`:

```typescript
import { configureStore } from '@reduxjs/toolkit'

export const store = configureStore({
  reducer: {},
})
```

**MUI theme** — Create `src/theme.ts`:

```typescript
import { createTheme } from '@mui/material/styles'

const theme = createTheme({
  palette: {
    mode: 'dark',
  },
})

export default theme
```

**Page components** — Create `src/pages/Home.tsx`:

```typescript
import ClownPuncher from '@/components/game/ClownPuncher'

const Home = () => <ClownPuncher />
export default Home
```

Create `src/pages/NotFound.tsx`:

```typescript
const NotFound = () => <div>404 - Page not found</div>
export default NotFound
```

**Stylesheets** — Create `src/App.css` (empty for now) and replace `src/index.css` with:

```css
@import "tailwindcss";
```

### Entry point: main.tsx

Create `src/main.tsx` to bootstrap the app with theme and state management:

```typescript
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { Provider } from 'react-redux'
import { ToastContainer } from 'react-toastify'
import 'react-toastify/dist/ReactToastify.css'

import App from '@/App.tsx'
import { store } from '@/store/store'
import theme from '@/theme.ts'
import { ThemeProvider } from '@mui/material/styles'

import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider theme={theme}>
      <Provider store={store}>
        <App />
        <ToastContainer />
      </Provider>
    </ThemeProvider>
  </StrictMode>
)
```
