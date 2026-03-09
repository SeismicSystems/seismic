---
description: Send encrypted write transactions to shielded contracts
icon: pen
---

# useShieldedWrite

{% hint style="info" %}
This hook wraps `useShieldedWriteContract` from seismic-react. The actual export name is `useShieldedWriteContract`.
{% endhint %}

Hook for sending shielded write transactions. Encrypts calldata before submission so transaction data is not visible on-chain.

```typescript
import { useShieldedWriteContract } from "seismic-react";
```

## Config

| Parameter      | Type     | Required | Description                             |
| -------------- | -------- | -------- | --------------------------------------- |
| `address`      | `Hex`    | Yes      | Contract address                        |
| `abi`          | `Abi`    | Yes      | Contract ABI                            |
| `functionName` | `string` | Yes      | Name of the nonpayable/payable function |
| `args`         | `array`  | No       | Arguments to pass to the function       |
| `gas`          | `bigint` | No       | Gas limit override                      |
| `gasPrice`     | `bigint` | No       | Gas price override                      |

## Return Type

| Property        | Type                        | Description                                 |
| --------------- | --------------------------- | ------------------------------------------- |
| `writeContract` | `() => Promise<Hex>`        | Function to execute the shielded write      |
| `write`         | `() => Promise<Hex>`        | Alias for writeContract                     |
| `isLoading`     | `boolean`                   | Whether a write is in progress              |
| `error`         | `Error \| null`             | Error from the most recent write            |
| `hash`          | `` `0x${string}` \| null `` | Transaction hash from last successful write |

---

## Usage

### Shielded token transfer

```typescript
import { useShieldedWriteContract } from 'seismic-react'

const abi = [
  {
    name: 'transfer',
    type: 'function',
    stateMutability: 'nonpayable',
    inputs: [
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'uint256' },
    ],
    outputs: [],
  },
] as const

function TransferToken() {
  const { writeContract, isLoading, error, hash } = useShieldedWriteContract({
    address: '0x1234567890abcdef1234567890abcdef12345678',
    abi,
    functionName: 'transfer',
    args: ['0xRecipientAddress...', 1000n],
  })

  return (
    <div>
      <button onClick={writeContract} disabled={isLoading}>
        {isLoading ? 'Sending...' : 'Transfer'}
      </button>
      {hash && <p>Transaction: {hash}</p>}
      {error && <p>Error: {error.message}</p>}
    </div>
  )
}
```

### Transaction hash tracking

The `hash` field updates after each successful write. Use it to link to a block explorer or track confirmation.

```typescript
import { useShieldedWriteContract } from 'seismic-react'
import { useEffect } from 'react'

function WriteWithTracking() {
  const { writeContract, hash, isLoading } = useShieldedWriteContract({
    address: CONTRACT_ADDRESS,
    abi,
    functionName: 'increment',
  })

  useEffect(() => {
    if (hash) {
      console.log('Transaction confirmed:', hash)
    }
  }, [hash])

  return (
    <div>
      <button onClick={writeContract} disabled={isLoading}>
        Increment
      </button>
      {hash && (
        <a href={`https://explorer.seismicdev.net/tx/${hash}`} target="_blank" rel="noreferrer">
          View on explorer
        </a>
      )}
    </div>
  )
}
```

### Loading and error state handling

```typescript
import { useShieldedWriteContract } from 'seismic-react'

function WriteWithStates() {
  const { writeContract, isLoading, error, hash } = useShieldedWriteContract({
    address: CONTRACT_ADDRESS,
    abi,
    functionName: 'setNumber',
    args: [42n],
  })

  return (
    <div>
      <button onClick={writeContract} disabled={isLoading}>
        {isLoading ? 'Encrypting & sending...' : 'Set Number'}
      </button>
      {isLoading && <p>Transaction in progress...</p>}
      {error && <p style={{ color: 'red' }}>Failed: {error.message}</p>}
      {hash && <p style={{ color: 'green' }}>Success: {hash}</p>}
    </div>
  )
}
```

### With gas override

Pass `gas` or `gasPrice` to override the automatic estimates:

```typescript
import { useShieldedWriteContract } from 'seismic-react'

function WriteWithGasOverride() {
  const { writeContract, isLoading } = useShieldedWriteContract({
    address: CONTRACT_ADDRESS,
    abi,
    functionName: 'expensiveOperation',
    gas: 500_000n,
    gasPrice: 20_000_000_000n,
  })

  return (
    <button onClick={writeContract} disabled={isLoading}>
      Execute
    </button>
  )
}
```

{% hint style="info" %}
Like `useSignedReadContract`, the write function is imperative -- you call `writeContract()` or `write()` explicitly. The hook does not auto-execute on mount or when arguments change.
{% endhint %}

## See Also

- [useSignedReadContract](useshieldedread.md) -- Perform signed, encrypted read calls
- [useShieldedContract](useshieldedcontract.md) -- Contract instance with both read and write methods
- [useShieldedWallet](useshieldedwallet.md) -- Access the underlying wallet client
- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Context provider required by this hook
- [Hooks Overview](./) -- Summary of all hooks
