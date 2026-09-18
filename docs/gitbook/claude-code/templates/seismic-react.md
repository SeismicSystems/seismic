# Template: Seismic React Client

Generate a complete React application with Seismic's shielded contract hooks for private blockchain interactions.

## Overview

This template creates a React app that:
1. Connects to Seismic Testnet via ShieldedWalletProvider
2. Reads public and shielded contract state
3. Writes public and shielded transactions
4. Handles wallet connection and transaction status

## Prerequisites

- Node.js 18+
- MetaMask or compatible wallet
- Seismic Testnet configured in wallet

## Template Prompt

```
Create a React application for interacting with Seismic shielded contracts.

Requirements:
1. Use Vite + React + TypeScript
2. Install and configure @seismic-systems/seismic-react and wagmi
3. Wrap the app with ShieldedWalletProvider (chain: seismicDevnet)
4. Show connection status and wallet address
5. Include examples for:
   - Public reads via wagmi's useReadContract
   - Shielded reads via useShieldedContract().read
   - Public writes via wagmi's useWriteContract
   - Shielded writes via useShieldedWriteContract
6. Display transaction hashes and status
7. Handle loading and error states
8. Clean, modern UI with Tailwind CSS

Contract ABI (example Counter):
[
  {
    "type": "function",
    "name": "getNumber",
    "inputs": [],
    "outputs": [{"name": "", "type": "uint256"}],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getShieldedNumber",
    "inputs": [],
    "outputs": [{"name": "", "type": "suint256"}],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "setNumber",
    "inputs": [{"name": "newNumber", "type": "uint256"}],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setShieldedNumber",
    "inputs": [{"name": "newNumber", "type": "suint256"}],
    "outputs": [],
    "stateMutability": "nonpayable"
  }
]

Contract address: [USER_PROVIDES_ADDRESS]
```

## Expected Output Structure

```
seismic-react-app/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── vite-env.d.ts
│   ├── config/
│   │   └── wagmi.ts
│   ├── contracts/
│   │   └── abi.ts
│   └── components/
│       ├── ConnectWallet.tsx
│       ├── PublicCounter.tsx
│       └── ShieldedCounter.tsx
└── README.md
```

## Key Code Patterns

### Wagmi + Seismic Provider Setup

```typescript
// src/config/wagmi.ts
import { http, createConfig } from 'wagmi'
import { seismicDevnet } from '@seismic-systems/seismic-react'

export const config = createConfig({
  chains: [seismicDevnet],
  transports: {
    [seismicDevnet.id]: http(),
  },
})
```

```tsx
// src/main.tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { WagmiProvider } from 'wagmi'
import { ShieldedWalletProvider } from '@seismic-systems/seismic-react'
import { config } from './config/wagmi'
import { seismicDevnet } from '@seismic-systems/seismic-react'
import App from './App'
import './index.css'

const queryClient = new QueryClient()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <ShieldedWalletProvider config={config} chain={seismicDevnet}>
          <App />
        </ShieldedWalletProvider>
      </QueryClientProvider>
    </WagmiProvider>
  </StrictMode>,
)
```

### Contract ABI

```typescript
// src/contracts/abi.ts
export const COUNTER_ADDRESS = '0x...' as const // user-provided

export const counterAbi = [
  {
    type: 'function',
    name: 'getNumber',
    inputs: [],
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getShieldedNumber',
    inputs: [],
    outputs: [{ name: '', type: 'suint256' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'setNumber',
    inputs: [{ name: 'newNumber', type: 'uint256' }],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'setShieldedNumber',
    inputs: [{ name: 'newNumber', type: 'suint256' }],
    outputs: [],
    stateMutability: 'nonpayable',
  },
] as const
```

### Public Reads and Writes (wagmi)

```tsx
import { useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi'
import { COUNTER_ADDRESS, counterAbi } from '../contracts/abi'

export function PublicCounter() {
  const { data: number, isLoading, error, refetch } = useReadContract({
    address: COUNTER_ADDRESS,
    abi: counterAbi,
    functionName: 'getNumber',
  })

  const { writeContract, data: hash, isPending, error: writeError } = useWriteContract()
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash })

  const setNumber = (value: bigint) => {
    writeContract({
      address: COUNTER_ADDRESS,
      abi: counterAbi,
      functionName: 'setNumber',
      args: [value],
    })
  }

  // ...render UI with number, loading/error, setNumber controls, hash/status
}
```

### Shielded Reads (`useShieldedContract`)

`useShieldedContract` returns a contract instance. Call shielded view methods through `read`, and check wallet/session status with `isShielded` / `isError`:

```tsx
import { useShieldedContract } from '@seismic-systems/seismic-react'
import { COUNTER_ADDRESS, counterAbi } from '../contracts/abi'

export function ShieldedCounter() {
  const { read, isShielded, isError } = useShieldedContract({
    abi: counterAbi,
    address: COUNTER_ADDRESS,
  })

  // Example: read.getShieldedNumber()
  // Gate UI on isShielded; surface isError when the shielded session is unavailable
}
```

### Shielded Writes (`useShieldedWriteContract`)

Pass contract config when creating the hook. The returned `writeContractAsync` only takes the call (`functionName` + `args`):

```tsx
import { useShieldedWriteContract } from '@seismic-systems/seismic-react'
import { COUNTER_ADDRESS, counterAbi } from '../contracts/abi'

export function ShieldedCounterWrite() {
  const { writeContractAsync, data: hash, isPending, error } = useShieldedWriteContract({
    address: COUNTER_ADDRESS,
    abi: counterAbi,
  })

  const setShieldedNumber = async (value: bigint) => {
    await writeContractAsync({
      functionName: 'setShieldedNumber',
      args: [value],
    })
  }

  // ...render UI with write controls, hash, pending/error states
}
```

## Common Customizations

### Different Contract

```
Modify the template to work with this ERC20-like contract instead:
[PASTE ABI]

Focus on balanceOf (read) and transfer (write) functions.
```

### Add Event Listening

```
Add event listening for NumberSet and ShieldedNumberSet events.
Display a live feed of recent events.
```

### Multi-Contract Dashboard

```
Create a dashboard that interacts with multiple contracts:
1. Counter contract at 0x...
2. Token contract at 0x...

Show balances and allow interactions with both.
```

## Troubleshooting Hints for Claude

If generation fails or produces incorrect code, remind Claude:

1. **`useShieldedRead` / `useShieldedWrite` do not exist** — use `useShieldedContract` for shielded reads and `useShieldedWriteContract` for shielded writes
2. Must wrap app with `ShieldedWalletProvider` (and typically `WagmiProvider` + `QueryClientProvider`)
3. Use `seismicDevnet` from `@seismic-systems/seismic-react` for the chain config
4. Shielded writes: contract `address`/`abi` go into `useShieldedWriteContract({...})`; `writeContractAsync` only gets `functionName`/`args`
5. Public reads/writes use standard wagmi hooks (`useReadContract`, `useWriteContract`)
6. Always handle `isPending` / loading and `error` states
7. Contract address must be a valid checksummed address

## Related Documentation

- [Building a Frontend](https://docs.seismic.systems/building-with-seismic/building-a-frontend) — seismic-react setup and hooks
- [seismic-react on npm](https://www.npmjs.com/package/@seismic-systems/seismic-react)
- [Contract Compatibility](https://docs.seismic.systems/building-with-seismic/contract-compatibility)
- [Network Information](https://docs.seismic.systems/developers/network-information)
