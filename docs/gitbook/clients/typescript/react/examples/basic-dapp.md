---
description: Complete minimal dApp with shielded writes and signed reads
icon: play
---

# Basic dApp

This example builds a complete minimal dApp that connects a wallet, sends a shielded write transaction, and performs a signed read -- all using seismic-react hooks.

## What You'll Build

1. **Provider setup** with RainbowKit + Seismic
2. **Wallet connection** via RainbowKit's `ConnectButton`
3. **Shielded write** to a contract (encrypted calldata)
4. **Signed read** from a contract (authenticated query)

## Prerequisites

- Node.js 18+
- A [WalletConnect](https://cloud.walletconnect.com/) project ID

Install dependencies:

```bash
npm install seismic-react seismic-viem wagmi viem @rainbow-me/rainbowkit @tanstack/react-query
```

## Step 1: wagmi Config

Create the wagmi configuration with the Seismic testnet chain:

```typescript
// config.ts
import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { seismicTestnet } from "seismic-react/rainbowkit";

export const config = getDefaultConfig({
  appName: "Seismic Basic dApp",
  projectId: "YOUR_WALLETCONNECT_PROJECT_ID",
  chains: [seismicTestnet],
});
```

{% hint style="warning" %}
Replace `YOUR_WALLETCONNECT_PROJECT_ID` with your actual project ID from [WalletConnect Cloud](https://cloud.walletconnect.com/).
{% endhint %}

## Step 2: Provider Wrapper

Wrap your app with the required providers. `ShieldedWalletProvider` must be nested inside the wagmi and React Query providers:

```typescript
// providers.tsx
'use client'

import { RainbowKitProvider } from '@rainbow-me/rainbowkit'
import { WagmiProvider } from 'wagmi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ShieldedWalletProvider } from 'seismic-react'
import { config } from './config'
import '@rainbow-me/rainbowkit/styles.css'

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

## Step 3: Contract Interaction Component

Create a component that uses `useShieldedWriteContract` and `useSignedReadContract` to interact with a shielded counter contract:

```typescript
// ShieldedCounter.tsx
'use client'

import { useShieldedWriteContract, useSignedReadContract } from 'seismic-react'
import { useState } from 'react'

const CONTRACT_ADDRESS = '0x...' as const
const ABI = [
  {
    name: 'increment',
    type: 'function',
    stateMutability: 'nonpayable',
    inputs: [],
    outputs: [],
  },
  {
    name: 'getCount',
    type: 'function',
    stateMutability: 'view',
    inputs: [],
    outputs: [{ name: '', type: 'uint256' }],
  },
] as const

export function ShieldedCounter() {
  const [count, setCount] = useState<string | null>(null)

  const { writeContract, isLoading: isWriting, hash, error: writeError } = useShieldedWriteContract({
    address: CONTRACT_ADDRESS,
    abi: ABI,
    functionName: 'increment',
  })

  const { signedRead, isLoading: isReading, error: readError } = useSignedReadContract({
    address: CONTRACT_ADDRESS,
    abi: ABI,
    functionName: 'getCount',
  })

  const handleIncrement = async () => {
    try {
      await writeContract()
    } catch (err) {
      console.error('Shielded write failed:', err)
    }
  }

  const handleRead = async () => {
    try {
      const result = await signedRead()
      setCount(result?.toString() ?? 'unknown')
    } catch (err) {
      console.error('Signed read failed:', err)
    }
  }

  return (
    <div>
      <h2>Shielded Counter</h2>

      <button onClick={handleIncrement} disabled={isWriting}>
        {isWriting ? 'Sending...' : 'Increment (Shielded Write)'}
      </button>
      {hash && <p>Tx hash: {hash}</p>}
      {writeError && <p>Write error: {writeError.message}</p>}

      <button onClick={handleRead} disabled={isReading}>
        {isReading ? 'Reading...' : 'Get Count (Signed Read)'}
      </button>
      {count !== null && <p>Count: {count}</p>}
      {readError && <p>Read error: {readError.message}</p>}
    </div>
  )
}
```

{% hint style="info" %}
Replace `CONTRACT_ADDRESS` with the address of a deployed shielded contract on Seismic testnet. The ABI above is for a simple counter -- adapt it to match your contract.
{% endhint %}

## Step 4: App Component

Combine the wallet connection button with the counter component. The counter only renders once the shielded wallet is ready:

```typescript
// App.tsx
import { ConnectButton } from '@rainbow-me/rainbowkit'
import { useShieldedWallet } from 'seismic-react'
import { ShieldedCounter } from './ShieldedCounter'

export function App() {
  const { loaded, error } = useShieldedWallet()

  return (
    <div>
      <h1>Seismic Basic dApp</h1>
      <ConnectButton />

      {error && <p>Wallet error: {error}</p>}
      {loaded ? (
        <ShieldedCounter />
      ) : (
        <p>Connect your wallet to interact with shielded contracts.</p>
      )}
    </div>
  )
}
```

## What's Happening

1. **RainbowKit** handles wallet connection and chain switching
2. **ShieldedWalletProvider** automatically creates shielded clients when a wallet connects, performing the ECDH key exchange with the TEE
3. **useShieldedWriteContract** encrypts calldata before sending the transaction, ensuring on-chain privacy
4. **useSignedReadContract** attaches a signature proving caller identity, allowing the contract to return private data only to authorized readers

```
User connects wallet
        |
        v
ShieldedWalletProvider creates shielded clients (ECDH with TEE)
        |
        v
useShieldedWriteContract    useSignedReadContract
  - encrypts calldata         - signs the read request
  - sends TxSeismic           - node verifies identity
  - returns tx hash           - returns decrypted result
```

## Next Steps

- [Hooks Reference](../hooks/) - Full API for all hooks
- [Wallet Guides](../wallet-guides/) - Use AppKit or Privy instead of RainbowKit
- [Chains](../chains/) - Configure Seismic testnet or local Sanvil

## See Also

- [ShieldedWalletProvider](../shielded-wallet-provider.md) - Provider configuration options
- [useShieldedWriteContract](../hooks/useshieldedwrite.md) - Shielded write hook API
- [useSignedReadContract](../hooks/useshieldedread.md) - Signed read hook API
- [Installation](../installation.md) - Full dependency setup
