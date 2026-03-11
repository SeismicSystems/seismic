---
description: Connect your SRC20 contract to a React frontend with seismic-react
icon: browser
---

# Building the Frontend

This chapter connects the SRC20 contract to a React frontend using `seismic-react`, which composes with [wagmi](https://wagmi.sh/) to provide shielded reads, shielded writes, and encrypted communication out of the box. _Estimated time: \~25 minutes._

## Overview

By the end of this chapter you will have a React application that can:

- Connect a wallet through a `ShieldedWalletProvider`
- Display the user's shielded balance (via signed reads)
- Transfer tokens (via shielded writes)
- Listen for Transfer events and decrypt the encrypted amounts

The patterns here mirror standard wagmi usage. If you have built a dApp with wagmi before, the `seismic-react` equivalents will feel familiar.

## Setup

Install the required packages:

```bash
npm install seismic-viem seismic-react viem wagmi @tanstack/react-query
```

### Configure the ShieldedWalletProvider

The `ShieldedWalletProvider` wraps your application and provides the shielded wallet context to all child components. It works alongside wagmi's standard provider:

```tsx
import { ShieldedWalletProvider } from "seismic-react";
import { WagmiProvider, createConfig, http } from "wagmi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { seismicTestnet } from "seismic-viem";

const config = createConfig({
  chains: [seismicTestnet],
  transports: {
    [seismicTestnet.id]: http("https://gcp-1.seismictest.net/rpc"),
  },
});

const queryClient = new QueryClient();

function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <ShieldedWalletProvider config={config}>
          <TokenDashboard />
        </ShieldedWalletProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
}
```

The `ShieldedWalletProvider` handles the cryptographic setup needed for Seismic transactions -- deriving encryption keys, managing the TEE public key, and signing shielded requests.

## Connecting to the contract

Use the `useShieldedContract` hook to get a contract instance that supports shielded reads and writes:

```typescript
import { useShieldedContract } from "seismic-react";
import { src20Abi } from "./abi";

const SRC20_ADDRESS = "0x1234..."; // Your deployed contract address

function useToken() {
  const contract = useShieldedContract({
    address: SRC20_ADDRESS,
    abi: src20Abi,
  });

  return contract;
}
```

This hook returns a contract instance bound to the currently connected shielded wallet. All reads are signed reads and all writes are shielded writes.

## Reading balance (signed read)

Use the `useSignedReadContract` hook to query the user's balance. This sends a signed read under the hood, so the contract can verify `msg.sender` and the response is encrypted:

```typescript
import { useSignedReadContract } from 'seismic-react';
import { useAccount } from 'wagmi';
import { useEffect, useState } from 'react';
import { formatEther } from 'viem';

function BalanceDisplay() {
  const { address } = useAccount();
  const [balance, setBalance] = useState<bigint | null>(null);

  const { signedRead, isLoading, error } = useSignedReadContract({
    address: SRC20_ADDRESS,
    abi: src20Abi,
    functionName: 'balanceOf',
    args: [address],
  });

  useEffect(() => {
    if (signedRead) {
      signedRead().then(setBalance);
    }
  }, [signedRead]);

  if (isLoading) return <p>Loading balance...</p>;
  if (error) return <p>Error loading balance</p>;

  return (
    <div>
      <h2>Your Balance</h2>
      <p>{formatEther(balance ?? 0n)} SRC</p>
    </div>
  );
}
```

The `useSignedReadContract` hook handles the full signed-read flow: signing the request with the user's key, sending it to `eth_call`, and decrypting the encrypted response. It returns a `signedRead` function that you call imperatively to perform the read.

## Transferring tokens (shielded write)

Use the `useShieldedWriteContract` hook to send a shielded transfer. The calldata is encrypted before leaving the user's machine:

```typescript
import { useShieldedWriteContract } from 'seismic-react';
import { parseEther } from 'viem';
import { useState } from 'react';

function TransferForm() {
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');

  const { writeContract, isLoading, error, hash } = useShieldedWriteContract();

  const handleTransfer = () => {
    writeContract({
      address: SRC20_ADDRESS,
      abi: src20Abi,
      functionName: 'transfer',
      args: [recipient, parseEther(amount)],
    });
  };

  return (
    <div>
      <h2>Transfer Tokens</h2>
      <input
        type="text"
        placeholder="Recipient address (0x...)"
        value={recipient}
        onChange={(e) => setRecipient(e.target.value)}
      />
      <input
        type="text"
        placeholder="Amount"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
      />
      <button onClick={handleTransfer} disabled={isLoading}>
        {isLoading ? 'Sending...' : 'Transfer'}
      </button>
      {hash && <p>Transfer submitted: {hash}</p>}
      {error && <p>Error: {error.message}</p>}
    </div>
  );
}
```

When `transfer()` is called, the `seismic-react` library:

1. Fetches the TEE public key from the node.
2. Derives a shared secret via ECDH.
3. Encrypts the calldata (including the recipient and amount) with AEAD.
4. Wraps everything in a Seismic transaction (type `0x4A`).
5. Broadcasts the encrypted transaction.

The user sees a standard wallet confirmation prompt. The privacy happens automatically under the hood.

## Decrypting events

If your contract uses encrypted events (from the [Encrypted Events](encrypted-events.md) chapter), you can listen for them and decrypt the amounts off-chain:

Decrypt events client-side using the ECDH shared secret and AES-GCM decryption. See the [seismic-viem precompiles documentation](../../clients/typescript/viem/precompiles.md) for the cryptographic primitives.

## Complete example

Here is the full dashboard component that ties everything together:

```tsx
import { useAccount } from "wagmi";
import { BalanceDisplay } from "./BalanceDisplay";
import { TransferForm } from "./TransferForm";

function TokenDashboard() {
  const { address, isConnected } = useAccount();

  if (!isConnected) {
    return (
      <div>
        <h1>SRC20 Token Dashboard</h1>
        <p>Connect your wallet to get started.</p>
        {/* Add your wallet connect button here (e.g., RainbowKit, Privy, AppKit) */}
      </div>
    );
  }

  return (
    <div>
      <h1>SRC20 Token Dashboard</h1>
      <p>Connected: {address}</p>
      <BalanceDisplay />
      <TransferForm />
    </div>
  );
}
```

{% hint style="info" %}
In a production application, you should not hardcode private keys. Use a wallet provider (such as RainbowKit, Privy, or AppKit) to manage keys securely. See the [Wallet Guides](../../clients/typescript/react/wallet-guides/README.md) for integration details.
{% endhint %}

## Next steps

You now have a complete SRC20 token: a private ERC20 with shielded balances, encrypted events, signed reads, compliance access control, and a React frontend.

From here, you can:

- **Explore the client library docs** -- The [Client Libraries section](../../clients/README.md) has detailed API references for `seismic-viem` and `seismic-react`, including all available hooks, wallet client methods, and precompile utilities.
- **Add wallet integration** -- See the [Wallet Guides](../../clients/typescript/react/wallet-guides/README.md) for step-by-step instructions on integrating RainbowKit, Privy, or AppKit with `seismic-react`.
- **Deploy to testnet** -- See the [deploy section](../../reference/migrating-from-ethereum.md#step-8-deploy) for deploying your SRC20 to a live Seismic network.
- **Extend the contract** -- Consider adding features like burn functions or governance mechanisms.
