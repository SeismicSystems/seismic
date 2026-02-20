---
description: Access shielded public and wallet clients from context
icon: wallet
---

# useShieldedWallet

Hook that consumes the `ShieldedWalletProvider` context. Returns the shielded public client, wallet client, connected address, error state, and a loaded flag indicating whether initialization is complete.

```typescript
import { useShieldedWallet } from "seismic-react";
```

## Return Type

| Property       | Type                           | Description                                         |
| -------------- | ------------------------------ | --------------------------------------------------- |
| `publicClient` | `ShieldedPublicClient \| null` | Shielded public client for reads                    |
| `walletClient` | `ShieldedWalletClient \| null` | Shielded wallet client for writes                   |
| `address`      | `Hex \| null`                  | Connected wallet address                            |
| `error`        | `string \| null`               | Error message if initialization failed              |
| `loaded`       | `boolean`                      | Whether the wallet client has finished initializing |

---

## Usage

### Basic

```typescript
import { useShieldedWallet } from 'seismic-react'

function WalletInfo() {
  const { address, walletClient, publicClient } = useShieldedWallet()

  if (!walletClient) return <div>Connect your wallet</div>

  return <div>Connected: {address}</div>
}
```

### Handling loading state

The `loaded` flag is `false` until the shielded clients have been fully initialized. Use it to avoid rendering components that depend on the wallet client before it is ready.

```typescript
import { useShieldedWallet } from 'seismic-react'

function App() {
  const { loaded, walletClient, address } = useShieldedWallet()

  if (!loaded) {
    return <div>Initializing shielded wallet...</div>
  }

  if (!walletClient) {
    return <div>Please connect your wallet</div>
  }

  return <div>Wallet ready: {address}</div>
}
```

### Error handling

If initialization fails (for example, the node is unreachable or the wallet connector is incompatible), the `error` field contains the error message.

```typescript
import { useShieldedWallet } from 'seismic-react'

function WalletStatus() {
  const { loaded, error, walletClient } = useShieldedWallet()

  if (!loaded) return <div>Loading...</div>
  if (error) return <div>Error: {error}</div>
  if (!walletClient) return <div>Not connected</div>

  return <div>Wallet connected</div>
}
```

### Accessing clients for direct seismic-viem calls

The returned `publicClient` and `walletClient` are full seismic-viem client instances. You can use them directly for operations not covered by the higher-level hooks.

```typescript
import { useShieldedWallet } from 'seismic-react'

function DirectClientUsage() {
  const { publicClient, walletClient } = useShieldedWallet()

  async function getBlockNumber() {
    if (!publicClient) return
    const block = await publicClient.getBlockNumber()
    console.log('Current block:', block)
  }

  async function sendRawTransaction() {
    if (!walletClient) return
    const hash = await walletClient.sendTransaction({
      to: '0x...',
      value: 0n,
    })
    console.log('Transaction hash:', hash)
  }

  return (
    <div>
      <button onClick={getBlockNumber}>Get Block</button>
      <button onClick={sendRawTransaction}>Send Tx</button>
    </div>
  )
}
```

{% hint style="warning" %}
This hook must be used within a `ShieldedWalletProvider`. It will throw if called outside the provider tree.
{% endhint %}

## See Also

- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Context provider that supplies the shielded clients
- [useShieldedContract](useshieldedcontract.md) -- Create a contract instance using the wallet client
- [useShieldedWriteContract](useshieldedwrite.md) -- Send encrypted write transactions
- [useSignedReadContract](useshieldedread.md) -- Perform signed, encrypted read calls
- [Hooks Overview](./) -- Summary of all hooks
