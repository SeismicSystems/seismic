---
description: Deploy SRC20 tokens from a browser using the SRC20 Factory web interface
icon: browser
---

# Web Interface

The SRC20 Factory ships a React web GUI (`packages/web`) for deploying tokens directly from a browser wallet, no CLI or code required.

## Running the web app

```bash
cd packages/web
bun dev
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

## Building your own React frontend

The web app's token deployment logic is built on `@seismic/src20-sdk` and `seismic-react`. You can replicate the pattern in your own app. Below is the complete hook from `packages/web/src/hooks/useCreateToken.ts` you can adapt:

```tsx
import { useState } from "react";
import { useShieldedWallet } from "seismic-react";
import { createToken } from "@seismic/src20-sdk";
import type { CreateTokenResult } from "@seismic/src20-sdk";

interface UseCreateTokenParams {
  name: string;
  symbol: string;
  initialSupply: string; // whole tokens as a string; multiplied by 10¹⁸ internally
}

interface UseCreateTokenReturn {
  deploy: () => Promise<void>;
  isLoading: boolean;
  error: string | null;
  result: CreateTokenResult | null;
}

export function useCreateToken(
  params: UseCreateTokenParams,
): UseCreateTokenReturn {
  const { walletClient } = useShieldedWallet();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<CreateTokenResult | null>(null);

  const deploy = async () => {
    if (!walletClient) {
      setError("Wallet not connected");
      return;
    }

    setIsLoading(true);
    setError(null);
    setResult(null);

    try {
      const supplyBigInt =
        BigInt(params.initialSupply || "0") * BigInt(10 ** 18);
      const tokenResult = await createToken(walletClient, {
        name: params.name,
        symbol: params.symbol,
        initialSupply: supplyBigInt,
      });
      setResult(tokenResult);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Something went wrong");
    } finally {
      setIsLoading(false);
    }
  };

  return { deploy, isLoading, error, result };
}
```

Usage:

```tsx
function DeployButton() {
  const { deploy, isLoading, error, result } = useCreateToken({
    name: "My Private Token",
    symbol: "MPT",
    initialSupply: "1000000",
  });

  return (
    <div>
      <button onClick={deploy} disabled={isLoading}>
        {isLoading ? "Deploying..." : "Deploy Token"}
      </button>
      {error && <p>Error: {error}</p>}
      {result && <p>Token: {result.tokenAddress}</p>}
    </div>
  );
}
```

## Wagmi configuration

The web app uses wagmi with MetaMask on Seismic testnet:

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
