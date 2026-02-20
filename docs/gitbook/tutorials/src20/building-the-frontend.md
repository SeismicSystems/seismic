---
description: Connect your SRC20 contract to a React frontend with seismic-react
icon: browser
---

# Building the Frontend

This chapter connects the SRC20 contract to a React frontend using `seismic-react`, which composes with wagmi to provide shielded reads, shielded writes, and encrypted communication out of the box. _Estimated time: \~25 minutes._

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
import { seismicDevnet } from "seismic-viem/chains";

const config = createConfig({
  chains: [seismicDevnet],
  transports: {
    [seismicDevnet.id]: http("https://rpc-devnet.seismic.systems"),
  },
});

const queryClient = new QueryClient();

function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <ShieldedWalletProvider>
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

Use the `useShieldedRead` hook to query the user's balance. This sends a signed read under the hood, so the contract can verify `msg.sender` and the response is encrypted:

```typescript
import { useShieldedRead } from 'seismic-react';
import { useAccount } from 'wagmi';
import { formatEther } from 'viem';

function BalanceDisplay() {
  const { address } = useAccount();

  const { data: balance, isLoading, error } = useShieldedRead({
    address: SRC20_ADDRESS,
    abi: src20Abi,
    functionName: 'getBalance',
    args: [address],
    // Automatically refetches when a new block is produced
    watch: true,
  });

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

The `useShieldedRead` hook handles the full signed-read flow: signing the request with the user's key, sending it to `eth_call`, and decrypting the encrypted response. From the component's perspective, it behaves like a standard `useReadContract` from wagmi.

## Transferring tokens (shielded write)

Use the `useShieldedWrite` hook to send a shielded transfer. The calldata is encrypted before leaving the user's machine:

```typescript
import { useShieldedWrite } from 'seismic-react';
import { parseEther } from 'viem';
import { useState } from 'react';

function TransferForm() {
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');

  const { write: transfer, isLoading, isSuccess, error } = useShieldedWrite({
    address: SRC20_ADDRESS,
    abi: src20Abi,
    functionName: 'transfer',
  });

  const handleTransfer = () => {
    transfer({
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
      {isSuccess && <p>Transfer successful.</p>}
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

The user sees a standard wallet confirmation prompt. The privacy is handled transparently.

## Decrypting events

If your contract uses encrypted events (from the [Encrypted Events](encrypted-events.md) chapter), you can listen for them and decrypt the amounts off-chain:

```typescript
import { useEffect, useState } from 'react';
import { parseAbiItem } from 'viem';
import { usePublicClient } from 'wagmi';
import { decryptEventData } from 'seismic-viem';

interface TransferEvent {
  from: string;
  to: string;
  amount: string;
  blockNumber: bigint;
}

function TransactionHistory({ userPrivateKey, contractPublicKey }: {
  userPrivateKey: `0x${string}`;
  contractPublicKey: `0x${string}`;
}) {
  const publicClient = usePublicClient();
  const [transfers, setTransfers] = useState<TransferEvent[]>([]);

  useEffect(() => {
    async function fetchEvents() {
      const logs = await publicClient.getLogs({
        address: SRC20_ADDRESS,
        event: parseAbiItem(
          'event Transfer(address indexed from, address indexed to, bytes encryptedAmount)'
        ),
        fromBlock: 0n,
      });

      const decrypted: TransferEvent[] = [];

      for (const log of logs) {
        try {
          const amount = await decryptEventData({
            encryptedData: log.args.encryptedAmount,
            privateKey: userPrivateKey,
            contractPublicKey,
            context: 'src20-transfer-event',
          });

          decrypted.push({
            from: log.args.from,
            to: log.args.to,
            amount: formatEther(amount),
            blockNumber: log.blockNumber,
          });
        } catch {
          // Could not decrypt -- this event was not encrypted for us
        }
      }

      setTransfers(decrypted);
    }

    fetchEvents();
  }, [publicClient, userPrivateKey, contractPublicKey]);

  return (
    <div>
      <h2>Transaction History</h2>
      {transfers.length === 0 && <p>No transfers found.</p>}
      <ul>
        {transfers.map((tx, i) => (
          <li key={i}>
            Block {tx.blockNumber.toString()}: {tx.from} sent {tx.amount} SRC to {tx.to}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

The `decryptEventData` call will throw for events that were not encrypted to the current user's key. The `try/catch` handles this gracefully -- you simply skip events you cannot decrypt.

## Complete example

Here is the full dashboard component that ties everything together:

```tsx
import { useAccount } from "wagmi";
import { formatEther } from "viem";

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
      <TransactionHistory
        userPrivateKey={USER_PRIVATE_KEY}
        contractPublicKey={CONTRACT_PUBLIC_KEY}
      />
    </div>
  );
}
```

{% hint style="info" %}
In a production application, you should not hardcode private keys. Use a wallet provider (such as RainbowKit, Privy, or AppKit) to manage keys securely. See the [Wallet Guides](../../../clients/seismic-react/wallet-guides/) for integration details.
{% endhint %}

## Next steps

You now have a complete SRC20 token: a private ERC20 with shielded balances, encrypted events, signed reads, compliance access control, and a React frontend.

From here, you can:

- **Explore the client library docs** -- The [Client Libraries section](/broken/pages/nQsSEPZ1IbCEkrtaRNSq) has detailed API references for `seismic-viem` and `seismic-react`, including all available hooks, wallet client methods, and precompile utilities.
- **Add wallet integration** -- See the [Wallet Guides](../../../clients/seismic-react/wallet-guides/) for step-by-step instructions on integrating RainbowKit, Privy, or AppKit with `seismic-react`.
- **Deploy to testnet** -- The [Networks page](../../../reference/networks.md) has network configuration and faucet information for deploying your SRC20 to a live Seismic network.
- **Extend the contract** -- Consider adding features like shielded `saddress` for the recipient (hiding who receives tokens), burn functions, or governance mechanisms.
