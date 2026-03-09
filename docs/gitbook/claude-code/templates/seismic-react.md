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
- **Signed reads**: `eth_call` zeroes `msg.sender` on Seismic. Hooks like `useShieldedRead` handle this.
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
  useShieldedRead,
  useShieldedWrite,
} from "seismic-react";
```

## Core Patterns

### Wrap your app with ShieldedWalletProvider

```tsx
import { ShieldedWalletProvider } from "seismic-react";

function App() {
  return (
    <ShieldedWalletProvider>
      {/* Your app components */}
      <MyDapp />
    </ShieldedWalletProvider>
  );
}
```

### Access the shielded wallet

```tsx
import { useShieldedWallet } from "seismic-react";

function MyComponent() {
  const { walletClient, isConnected, address } = useShieldedWallet();

  if (!isConnected) return <p>Connect your wallet</p>;
  return <p>Connected: {address}</p>;
}
```

### Create a shielded contract instance

```tsx
import { useShieldedContract } from "seismic-react";

function MyComponent() {
  const contract = useShieldedContract({
    abi: myContractAbi,
    address: "0xCONTRACT_ADDRESS",
  });

  // contract.read.* for signed reads
  // contract.write.* for shielded writes
}
```

### Read shielded data (signed read)

```tsx
import { useShieldedRead } from "seismic-react";

function BalanceDisplay({ userAddress }: { userAddress: `0x${string}` }) {
  const { data: balance, isLoading } = useShieldedRead({
    abi: myContractAbi,
    address: "0xCONTRACT_ADDRESS",
    functionName: "getBalance",
    args: [userAddress],
  });

  if (isLoading) return <p>Loading...</p>;
  return <p>Balance: {balance?.toString()}</p>;
}
```

### Write shielded data (encrypted transaction)

```tsx
import { useShieldedWrite } from "seismic-react";

function TransferButton() {
  const { write, isLoading } = useShieldedWrite({
    abi: myContractAbi,
    address: "0xCONTRACT_ADDRESS",
    functionName: "transfer",
  });

  return (
    <button
      onClick={() => write({ args: [recipientAddress, amount] })}
      disabled={isLoading}
    >
      Transfer
    </button>
  );
}
```

### Wallet integration: RainbowKit

```tsx
import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { ShieldedWalletProvider } from "seismic-react";
import { WagmiProvider } from "wagmi";

function App() {
  return (
    <WagmiProvider config={wagmiConfig}>
      <RainbowKitProvider>
        <ShieldedWalletProvider>
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
import { ShieldedWalletProvider } from "seismic-react";

function App() {
  return (
    <PrivyProvider appId="YOUR_PRIVY_APP_ID">
      <ShieldedWalletProvider>
        <MyDapp />
      </ShieldedWalletProvider>
    </PrivyProvider>
  );
}
```

## Common Mistakes

1. **Using standard wagmi hooks** — `useContractRead`/`useContractWrite` from wagmi won't encrypt calldata or sign reads. Use `useShieldedRead`/`useShieldedWrite` from `seismic-react`.
2. **Forgetting the ShieldedWalletProvider** — All `useShielded*` hooks require `ShieldedWalletProvider` in the component tree. Wrap it around your app.
3. **Using EIP-1559 gas params** — Seismic uses legacy gas. Do NOT pass `maxFeePerGas`/`maxPriorityFeePerGas` to write hooks.
4. **Wrong import path** — The package is `seismic-react`, not `@seismic/react` or `seismic-viem/react`.
5. **Reading shielded data without wallet connection** — Signed reads require a connected wallet (they need the user's private key to sign). Show a "connect wallet" prompt first.

## Networks

| Network        | Chain ID | RPC URL                             |
| -------------- | -------- | ----------------------------------- |
| Devnet         | 5124     | `https://node-2.seismicdev.net/rpc` |
| Devnet (WS)    | 5124     | `wss://node-2.seismicdev.net/ws`    |
| Local (sanvil) | 31337    | `http://127.0.0.1:8545`             |

Faucet: https://faucet-2.seismicdev.net/

## Links

- [seismic-react Installation](https://docs.seismic.systems/client-libraries/seismic-react/installation)
- [ShieldedWalletProvider](https://docs.seismic.systems/client-libraries/seismic-react/shielded-wallet-provider)
- [Hooks Reference](https://docs.seismic.systems/client-libraries/seismic-react/hooks/)
- [RainbowKit Guide](https://docs.seismic.systems/client-libraries/seismic-react/wallet-guides/rainbowkit)
- [Privy Guide](https://docs.seismic.systems/client-libraries/seismic-react/wallet-guides/privy)
- [GitHub: seismic-client](https://github.com/SeismicSystems/seismic-client)
````

## What this teaches Claude

- **Correct hook names and import paths** — Claude will use `useShieldedRead`/`useShieldedWrite` instead of standard wagmi hooks
- **Provider hierarchy** — Claude will wrap the app with `ShieldedWalletProvider` in the correct order relative to wallet connectors
- **Signed reads in components** — Claude will use shielded hooks that handle identity-proving reads
- **Wallet integration patterns** — Claude knows how to combine RainbowKit/Privy with Seismic's provider

## Customizing

After pasting the template:

- Replace `[Your Project Name]` with your project name
- Add your contract ABIs and addresses
- Specify which wallet connector you use (RainbowKit, Privy, AppKit) so Claude defaults to the right integration pattern
- Add your wagmi config setup if you have custom chain definitions
