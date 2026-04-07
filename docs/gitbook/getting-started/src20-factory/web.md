---
description: Deploy SRC20 tokens from a browser using the SRC20 Factory web interface
icon: browser
---

# Web Interface

The SRC20 Factory ships a React web GUI for deploying tokens directly from a browser wallet, no CLI or code required.

## Deploying a token

1. Open the web app and connect MetaMask
2. Switch to the Seismic testnet (chain ID 5124)
3. Fill in the token name, symbol, and initial supply
4. Click **Deploy** and confirm the transaction in your wallet
5. The deployed token address and transaction hash appear on success

{% hint style="info" %}
Supply is entered in whole tokens. Entering `1000000` mints `1,000,000 × 10¹⁸` base units.
{% endhint %}

## Embedding in your own app

The web package exports a `useCreateToken` hook you can drop into any React app that already uses `seismic-react`.

### useCreateToken

```typescript
import { useCreateToken } from "@seismic/src20-web";
```

#### Signature

```typescript
function useCreateToken(params: UseCreateTokenParams): UseCreateTokenReturn;

interface UseCreateTokenParams {
  name: string;
  symbol: string;
  initialSupply: string; // whole tokens as a string; converted to bigint × 10¹⁸ internally
}

interface UseCreateTokenReturn {
  deploy: () => Promise<void>;
  isLoading: boolean;
  error: string | null;
  result: CreateTokenResult | null;
}
```

#### Example

```tsx
import { useCreateToken } from "@seismic/src20-web";

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

The hook calls `createToken` from `@seismic/src20-sdk` internally and surfaces human-readable error messages for common failure modes — wallet rejection, wrong network, insufficient funds, and encryption-related errors.

## Wagmi configuration

The web app is configured for MetaMask on Seismic testnet only:

```typescript
const wagmiConfig = createConfig({
  chains: [seismicTestnet],
  connectors: [injected({ target: "metaMask" })],
  transports: {
    [seismicTestnet.id]: http(),
  },
});
```

Other wallet connectors (WalletConnect, Coinbase, etc.) are not configured by default but can be added following the [wallet guides](../../clients/typescript/react/wallet-guides/README.md).
