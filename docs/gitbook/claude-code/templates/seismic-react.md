---
description: CLAUDE.md template for React frontend development with seismic-react
icon: react
---

# Seismic React

Use this template when your project uses `seismic-react` to build React frontends that interact with Seismic contracts. This SDK wraps `seismic-viem` with React hooks and providers, and integrates with wallet connectors like RainbowKit, Privy, and AppKit.

## The template

Copy the entire block below and save it as `CLAUDE.md` in your project root.

````markdown
# [Your Project Name]

## Seismic Overview

Seismic is an EVM-compatible L1 with on-chain privacy. Nodes run inside TEEs (Intel TDX). The Solidity compiler adds shielded types (`suint256`, `saddress`, `sbool`) that are invisible outside the TEE. Client libraries handle transaction encryption and signed reads automatically.

## Key Concepts

- **Shielded types**: `suint256`, `saddress`, `sbool` — on-chain private state, only readable via signed reads
- **TxSeismic (type 0x4A)**: Encrypts calldata before broadcast. The SDK handles this automatically.
- **Signed reads**: `eth_call` zeroes `msg.sender` on Seismic. Hooks like `useSignedReadContract` handle this.
- **Encryption pubkeys**: 33-byte compressed secp256k1 keys. The provider fetches and manages these.
- **Legacy gas**: Seismic transactions use `gas_price` + `gas_limit`, NOT EIP-1559.

## SDK: seismic-react

### Install

```bash
npm install seismic-react
# or
bun add seismic-react
```

### Key exports

```typescript
import {
  ShieldedWalletProvider,
  useShieldedWallet,
  useShieldedContract,
  useSignedReadContract,
  useShieldedWriteContract,
} from "seismic-react";
```

## Core Patterns

### Wrap your app with ShieldedWalletProvider

```tsx
import { ShieldedWalletProvider } from "seismic-react";
import { WagmiProvider, createConfig, http } from "wagmi";
import { seismicTestnet } from "seismic-viem";

const config = createConfig({
  chains: [seismicTestnet],
  transports: {
    [seismicTestnet.id]: http(),
  },
});

function App() {
  return (
    <WagmiProvider config={config}>
      <ShieldedWalletProvider config={config}>
        {/* Your app components */}
        <MyDapp />
      </ShieldedWalletProvider>
    </WagmiProvider>
  );
}
```

`ShieldedWalletProvider` requires a wagmi `config` prop — it cannot be used without one.

### Access the shielded wallet

```tsx
import { useShieldedWallet } from "seismic-react";

function MyComponent() {
  const { walletClient, address, loaded } = useShieldedWallet();

  if (!loaded) return <p>Loading...</p>;
  if (!walletClient) return <p>Connect your wallet</p>;
  return <p>Connected: {address}</p>;
}
```

### Create a shielded contract instance

```tsx
import { useShieldedContract } from "seismic-react";

function MyComponent() {
  const { contract, error } = useShieldedContract({
    abi: myContractAbi,
    address: "0xCONTRACT_ADDRESS",
  });

  if (error) return <p>Error: {error.message}</p>;
  if (!contract) return <p>Loading...</p>;

  // contract.read.* for signed reads
  // contract.write.* for shielded writes
}
```

### Read shielded data (signed read)

```tsx
import { useSignedReadContract } from "seismic-react";
import { useEffect, useState } from "react";

function BalanceDisplay({ userAddress }: { userAddress: `0x${string}` }) {
  const [balance, setBalance] = useState<bigint | null>(null);

  const { signedRead, isLoading, error } = useSignedReadContract({
    abi: myContractAbi,
    address: "0xCONTRACT_ADDRESS",
    functionName: "balanceOf",
    args: [userAddress],
  });

  useEffect(() => {
    signedRead().then((result) => {
      if (result !== undefined) setBalance(result as bigint);
    });
  }, [signedRead]);

  if (isLoading) return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;
  return <p>Balance: {balance?.toString()}</p>;
}
```

Unlike wagmi's `useReadContract`, `useSignedReadContract` does not auto-fetch — it returns a `signedRead` function you call imperatively.

### Write shielded data (encrypted transaction)

```tsx
import { useShieldedWriteContract } from "seismic-react";

function TransferButton({
  recipientAddress,
  amount,
}: {
  recipientAddress: `0x${string}`;
  amount: bigint;
}) {
  const { writeContract, isLoading, error, hash } = useShieldedWriteContract({
    abi: myContractAbi,
    address: "0xCONTRACT_ADDRESS",
    functionName: "transfer",
    args: [recipientAddress, amount],
  });

  return (
    <div>
      <button onClick={() => writeContract()} disabled={isLoading}>
        Transfer
      </button>
      {hash && <p>Transaction: {hash}</p>}
      {error && <p>Error: {error.message}</p>}
    </div>
  );
}
```

The write configuration — including `args` — goes into the hook itself; `writeContract()` is then called with no arguments.

### Wallet integration: RainbowKit

```tsx
import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { ShieldedWalletProvider } from "seismic-react";
import { WagmiProvider } from "wagmi";

function App() {
  return (
    <WagmiProvider config={wagmiConfig}>
      <RainbowKitProvider>
        <ShieldedWalletProvider config={wagmiConfig}>
          <MyDapp />
        </ShieldedWalletProvider>
      </RainbowKitProvider>
    </WagmiProvider>
  );
}
```

### Wallet integration: Privy

```tsx
import { PrivyProvider } from "@privy-io/react-auth";
import { WagmiProvider } from "@privy-io/wagmi";
import { ShieldedWalletProvider } from "seismic-react";

function App() {
  return (
    <PrivyProvider appId="YOUR_PRIVY_APP_ID">
      <WagmiProvider config={wagmiConfig}>
        <ShieldedWalletProvider config={wagmiConfig}>
          <MyDapp />
        </ShieldedWalletProvider>
      </WagmiProvider>
    </PrivyProvider>
  );
}
```

## Common Mistakes

1. **Using standard wagmi hooks** — `useContractRead`/`useContractWrite` from wagmi won't encrypt calldata or sign reads. Use `useSignedReadContract`/`useShieldedWriteContract` (or `useShieldedContract`) from `seismic-react`. There are no hooks named `useShieldedRead` or `useShieldedWrite`.
2. **Forgetting the ShieldedWalletProvider** — All shielded hooks require `ShieldedWalletProvider` (with its required `config` prop) in the component tree. Wrap it around your app.
3. **Using EIP-1559 gas params** — Seismic uses legacy gas. Do NOT pass `maxFeePerGas`/`maxPriorityFeePerGas` to write hooks.
4. **Wrong import path** — The package is `seismic-react`, not `@seismic/react` or `seismic-viem/react`.
5. **Reading shielded data without wallet connection** — Signed reads require a connected wallet (they need the user's private key to sign). Show a "connect wallet" prompt first.

## Networks

| Network        | Chain ID | RPC URL                             |
| -------------- | -------- | ----------------------------------- |
| Testnet         | 5124     | `https://testnet-1.seismictest.net/rpc` |
| Testnet (WS)    | 5124     | `wss://testnet-1.seismictest.net/ws`    |
| Local (sanvil) | 31337    | `http://127.0.0.1:8545`             |

Faucet: https://faucet.seismictest.net/

## Links

- [seismic-react Installation](https://docs.seismic.systems/clients/typescript/react/installation)
- [ShieldedWalletProvider](https://docs.seismic.systems/clients/typescript/react/shielded-wallet-provider)
- [Hooks Reference](https://docs.seismic.systems/clients/typescript/react/hooks/)
- [RainbowKit Guide](https://docs.seismic.systems/clients/typescript/react/wallet-guides/rainbowkit)
- [Privy Guide](https://docs.seismic.systems/clients/typescript/react/wallet-guides/privy)
- [GitHub: seismic (monorepo)](https://github.com/SeismicSystems/seismic)
````

## What this teaches Claude

- **Correct hook names and import paths** — Claude will use `useSignedReadContract`/`useShieldedWriteContract` instead of standard wagmi hooks
- **Provider hierarchy** — Claude will wrap the app with `ShieldedWalletProvider` in the correct order relative to wallet connectors
- **Signed reads in components** — Claude will use shielded hooks that handle identity-proving reads
- **Wallet integration patterns** — Claude knows how to combine RainbowKit/Privy with Seismic's provider

## Customizing

After pasting the template:

- Replace `[Your Project Name]` with your project name
- Add your contract ABIs and addresses
- Specify which wallet connector you use (RainbowKit, Privy, AppKit) so Claude defaults to the right integration pattern
- Add your wagmi config setup if you have custom chain definitions
