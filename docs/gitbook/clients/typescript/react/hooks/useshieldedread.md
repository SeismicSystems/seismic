---
description: Execute authenticated read calls on shielded contracts
icon: book-open-reader
---

# useShieldedRead

{% hint style="info" %}
This hook wraps `useSignedReadContract` from seismic-react. The actual export name is `useSignedReadContract`.
{% endhint %}

Hook for performing signed reads -- authenticated `eth_call` requests where the caller proves their identity. This allows contracts to return caller-specific shielded data (for example, a balance that depends on `msg.sender`).

```typescript
import { useSignedReadContract } from "seismic-react";
```

## Config

| Parameter      | Type                | Required | Description                            |
| -------------- | ------------------- | -------- | -------------------------------------- |
| `address`      | `` `0x${string}` `` | Yes      | Contract address                       |
| `abi`          | `Abi`               | Yes      | Contract ABI                           |
| `functionName` | `string`            | Yes      | Name of the view/pure function to call |
| `args`         | `array`             | No       | Arguments to pass to the function      |

## Return Type

| Property     | Type                 | Description                         |
| ------------ | -------------------- | ----------------------------------- |
| `signedRead` | `() => Promise<any>` | Function to execute the signed read |
| `read`       | `() => Promise<any>` | Alias for signedRead                |
| `isLoading`  | `boolean`            | Whether a read is in progress       |
| `error`      | `Error \| null`      | Error from the most recent read     |

---

## Usage

### Reading a shielded balance

```typescript
import { useSignedReadContract } from 'seismic-react'
import { useState } from 'react'

const abi = [
  {
    name: 'balanceOf',
    type: 'function',
    stateMutability: 'view',
    inputs: [],
    outputs: [{ name: '', type: 'uint256' }],
  },
] as const

function ShieldedBalance() {
  const [balance, setBalance] = useState<string | null>(null)

  const { signedRead, isLoading, error } = useSignedReadContract({
    address: '0x1234567890abcdef1234567890abcdef12345678',
    abi,
    functionName: 'balanceOf',
  })

  async function fetchBalance() {
    const result = await signedRead()
    if (result !== undefined) {
      setBalance(result.toString())
    }
  }

  return (
    <div>
      <button onClick={fetchBalance} disabled={isLoading}>
        {isLoading ? 'Reading...' : 'Get Balance'}
      </button>
      {balance && <p>Balance: {balance}</p>}
      {error && <p>Error: {error.message}</p>}
    </div>
  )
}
```

### Loading state handling

```typescript
import { useSignedReadContract } from 'seismic-react'

function ReadWithLoadingState() {
  const { signedRead, isLoading, error } = useSignedReadContract({
    address: CONTRACT_ADDRESS,
    abi,
    functionName: 'getSecret',
  })

  return (
    <div>
      <button onClick={signedRead} disabled={isLoading}>
        {isLoading ? 'Fetching...' : 'Read Secret'}
      </button>
      {isLoading && <span>Please wait, signing and decrypting...</span>}
    </div>
  )
}
```

### Error handling

```typescript
import { useSignedReadContract } from 'seismic-react'

function ReadWithErrorHandling() {
  const { signedRead, error } = useSignedReadContract({
    address: CONTRACT_ADDRESS,
    abi,
    functionName: 'balanceOf',
  })

  async function handleRead() {
    try {
      const result = await signedRead()
      console.log('Result:', result)
    } catch (e) {
      console.error('Signed read failed:', e)
    }
  }

  return (
    <div>
      <button onClick={handleRead}>Read</button>
      {error && <p style={{ color: 'red' }}>Last error: {error.message}</p>}
    </div>
  )
}
```

{% hint style="info" %}
Unlike wagmi's `useReadContract` which auto-fetches on mount, `useSignedReadContract` returns a function you call imperatively. This is because signed reads require wallet interaction to prove caller identity.
{% endhint %}

## See Also

- [useShieldedWriteContract](useshieldedwrite.md) -- Send encrypted write transactions
- [useShieldedContract](useshieldedcontract.md) -- Contract instance with both read and write methods
- [useShieldedWallet](useshieldedwallet.md) -- Access the underlying wallet client
- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Context provider required by this hook
- [Hooks Overview](./) -- Summary of all hooks
