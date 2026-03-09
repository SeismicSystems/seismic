---
description: Get a ShieldedContract instance for reads and writes
icon: file-contract
---

# useShieldedContract

Hook that creates a `ShieldedContract` instance from seismic-viem's `getShieldedContract`. Provides a contract object with type-safe methods for both shielded writes and signed reads.

```typescript
import { useShieldedContract } from "seismic-react";
```

## Config

| Parameter | Type      | Required | Description      |
| --------- | --------- | -------- | ---------------- |
| `abi`     | `Abi`     | Yes      | Contract ABI     |
| `address` | `Address` | Yes      | Contract address |

## Return Type

| Property   | Type                       | Description                            |
| ---------- | -------------------------- | -------------------------------------- |
| `contract` | `ShieldedContract \| null` | The shielded contract instance         |
| `abi`      | `Abi`                      | The ABI passed in                      |
| `address`  | `Address`                  | The address passed in                  |
| `error`    | `Error \| null`            | Error if wallet client not initialized |

---

## Usage

### Basic

```typescript
import { useShieldedContract } from 'seismic-react'

const abi = [
  {
    name: 'balanceOf',
    type: 'function',
    stateMutability: 'view',
    inputs: [],
    outputs: [{ name: '', type: 'uint256' }],
  },
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

function MyContract() {
  const { contract, error } = useShieldedContract({
    abi,
    address: '0x1234567890abcdef1234567890abcdef12345678',
  })

  if (error) return <div>Error: {error.message}</div>
  if (!contract) return <div>Loading contract...</div>

  return <div>Contract ready</div>
}
```

### Using the contract for reads and writes

Once you have a `ShieldedContract` instance, call its methods directly for both signed reads and shielded writes:

```typescript
import { useShieldedContract } from 'seismic-react'
import { useState } from 'react'

function TokenActions() {
  const [balance, setBalance] = useState<bigint | null>(null)
  const { contract } = useShieldedContract({ abi, address: CONTRACT_ADDRESS })

  async function readBalance() {
    if (!contract) return
    const result = await contract.read.balanceOf()
    setBalance(result as bigint)
  }

  async function transfer() {
    if (!contract) return
    const hash = await contract.write.transfer(['0xRecipient...', 100n])
    console.log('Transfer tx:', hash)
  }

  return (
    <div>
      <button onClick={readBalance}>Check Balance</button>
      {balance !== null && <p>Balance: {balance.toString()}</p>}
      <button onClick={transfer}>Transfer</button>
    </div>
  )
}
```

### TypeScript ABI typing

For full type inference on contract methods, define your ABI with `as const`:

```typescript
const abi = [
  {
    name: "increment",
    type: "function",
    stateMutability: "nonpayable",
    inputs: [],
    outputs: [],
  },
  {
    name: "number",
    type: "function",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;

// TypeScript now infers the available methods and their argument/return types
const { contract } = useShieldedContract({ abi, address: "0x..." });
```

{% hint style="info" %}
`useShieldedContract` requires a connected wallet via `ShieldedWalletProvider`. The `contract` field is `null` until the wallet client is initialized.
{% endhint %}

## See Also

- [useShieldedWallet](useshieldedwallet.md) -- Access the underlying wallet and public clients
- [useShieldedWriteContract](useshieldedwrite.md) -- Send encrypted writes without a contract instance
- [useSignedReadContract](useshieldedread.md) -- Perform signed reads without a contract instance
- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Context provider required by this hook
- [Hooks Overview](./) -- Summary of all hooks
