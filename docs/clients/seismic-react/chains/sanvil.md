---
description: Local development chain configurations
icon: server
---

# Sanvil

Chain configuration for connecting to a locally-running Seismic Anvil (Sanvil) instance. `sanvil` is a RainbowKit-compatible chain object pre-configured for the default Sanvil endpoint.

## Configuration

| Property        | Value                   |
| --------------- | ----------------------- |
| Chain ID        | `31337`                 |
| Name            | `Sanvil`                |
| RPC (HTTP)      | `http://127.0.0.1:8545` |
| Native Currency | ETH (18 decimals)       |

## Import

```typescript
import { sanvil } from "seismic-react/rainbowkit";
```

## Usage

### With RainbowKit

```typescript
import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { sanvil } from "seismic-react/rainbowkit";

const config = getDefaultConfig({
  appName: "My App",
  projectId: "YOUR_PROJECT_ID",
  chains: [sanvil],
});
```

### With wagmi Config

```typescript
import { http, createConfig } from "wagmi";
import { sanvil } from "seismic-react/rainbowkit";

const config = createConfig({
  chains: [sanvil],
  transports: {
    [sanvil.id]: http(),
  },
});
```

## localSeismicDevnet

`localSeismicDevnet` is a separate chain configuration for connecting to a locally-running `seismic-reth` node started in `--dev` mode. Use this instead of `sanvil` when you are running a full seismic-reth node locally rather than a Sanvil instance.

```typescript
import { localSeismicDevnet } from "seismic-react/rainbowkit";

const config = getDefaultConfig({
  appName: "My App",
  projectId: "YOUR_PROJECT_ID",
  chains: [localSeismicDevnet],
});
```

{% hint style="info" %}
Use `sanvil` for Sanvil (Seismic Anvil) instances. Use `localSeismicDevnet` for seismic-reth nodes running with the `--dev` flag.
{% endhint %}

## Installing Sanvil

Sanvil is part of the Seismic Foundry toolchain. Install it with `sfoundryup`:

```bash
curl -L https://raw.githubusercontent.com/SeismicSystems/seismic-foundry/seismic/sfoundryup/install | bash
sfoundryup
```

Then start a local node:

```bash
sanvil
```

By default, Sanvil:

- Listens on `127.0.0.1:8545`
- Uses chain ID `31337`
- Pre-funds test accounts with ETH
- Provides instant block mining

## See Also

- [Chains Overview](./) - All supported chains
- [Seismic Testnet](seismic-testnet.md) - Public testnet configuration
- [createSeismicDevnet](create-seismic-devnet.md) - Custom chain factory
- [Wallet Guides](../wallet-guides/) - RainbowKit setup guides
