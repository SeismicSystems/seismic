---
description: Deploy SRC20 tokens from a browser using the SRC20 Factory web interface
icon: browser
---

# Web Interface

The SRC20 Factory ships a React web GUI (`packages/web`) for deploying tokens directly from a browser wallet, no CLI or code required.

## Running the web app

Clone the repo and start the dev server:

```bash
bun install
bun run dev:web
```

## Deploying a token

1. Open the web app and connect MetaMask
2. Switch to the Seismic testnet (chain ID 5124)
3. Fill in the token name, symbol, and initial supply
4. Click **Deploy** and confirm the transaction in your wallet
5. The deployed token address and transaction hash appear on success

{% hint style="info" %}
Supply is entered in whole tokens. Entering `1000000` mints `1,000,000 × 10¹⁸` base units.
{% endhint %}

## Wagmi configuration

The web app connects MetaMask to Seismic testnet via wagmi:

```typescript
import { createConfig, http } from "wagmi";
import { injected } from "wagmi/connectors";
import { seismicTestnet } from "seismic-viem";

export const wagmiConfig = createConfig({
  chains: [seismicTestnet],
  connectors: [injected({ target: "metaMask" })],
  transports: {
    [seismicTestnet.id]: http(),
  },
});
```

Other wallet connectors (WalletConnect, Coinbase, etc.) are not configured by default but can be added following the [wallet guides](../../clients/typescript/react/wallet-guides/README.md).
