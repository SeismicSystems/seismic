---
description: React hooks for shielded transactions and signed reads
icon: puzzle-piece
---

# Hooks

seismic-react provides four hooks that mirror wagmi's hook API but route through Seismic's shielded transport layer. Each hook wraps the corresponding seismic-viem client or contract method, handling encryption, signing, and decryption automatically.

## Hook Comparison

| seismic-react Hook         | wagmi Equivalent                 | Purpose                                   |
| -------------------------- | -------------------------------- | ----------------------------------------- |
| `useShieldedWallet`        | `useAccount` + `useWalletClient` | Access shielded public and wallet clients |
| `useShieldedContract`      | `useContract`                    | Get a ShieldedContract instance           |
| `useShieldedWriteContract` | `useWriteContract`               | Send encrypted write transactions         |
| `useSignedReadContract`    | `useReadContract`                | Execute authenticated read calls          |

{% hint style="info" %}
All hooks require `ShieldedWalletProvider` context. Calling any hook outside the provider tree will throw an error.
{% endhint %}

## Common Patterns

### Wallet check

Always verify the wallet is loaded before calling contract methods:

```typescript
const { walletClient, loaded } = useShieldedWallet()

if (!loaded) return <div>Loading wallet...</div>
if (!walletClient) return <div>Please connect your wallet</div>
```

### Loading states

The write and read hooks expose `isLoading` so you can disable buttons or show spinners:

```typescript
const { writeContract, isLoading } = useShieldedWriteContract({ address, abi, functionName: 'transfer', args })

return (
  <button onClick={writeContract} disabled={isLoading}>
    {isLoading ? 'Sending...' : 'Transfer'}
  </button>
)
```

### Error handling

Every hook returns an `error` field. Check it after operations complete:

```typescript
const { signedRead, error } = useSignedReadContract({
  address,
  abi,
  functionName: "balanceOf",
});

async function fetchBalance() {
  const result = await signedRead();
  if (error) {
    console.error("Read failed:", error.message);
  }
}
```

## Pages

| Page                                            | Description                                            |
| ----------------------------------------------- | ------------------------------------------------------ |
| [useShieldedWallet](useshieldedwallet.md)       | Access shielded wallet and public clients from context |
| [useShieldedContract](useshieldedcontract.md)   | Get a ShieldedContract instance for reads and writes   |
| [useShieldedWriteContract](useshieldedwrite.md) | Send encrypted write transactions                      |
| [useSignedReadContract](useshieldedread.md)     | Execute authenticated read calls                       |

## See Also

- [ShieldedWalletProvider](../shielded-wallet-provider.md) -- Context provider required by all hooks
- [Installation](../installation.md) -- Package setup and peer dependencies
- [Seismic React Overview](../) -- SDK architecture and quick start
