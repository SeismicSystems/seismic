---
description: Install seismic-viem and configure viem peer dependency
icon: download
---

# Installation

## Prerequisites

| Requirement | Version | Notes                                                                                                  |
| ----------- | ------- | ------------------------------------------------------------------------------------------------------ |
| Node.js     | 18+     | LTS recommended; install via [nvm](https://github.com/nvm-sh/nvm) or [nodejs.org](https://nodejs.org/) |
| viem        | 2.x     | Peer dependency -- installed alongside seismic-viem                                                    |

## Install

Install `seismic-viem` alongside its `viem` peer dependency using your preferred package manager:

{% tabs %}
{% tab title="npm" %}

```bash
npm install seismic-viem viem
```

{% endtab %}

{% tab title="yarn" %}

```bash
yarn add seismic-viem viem
```

{% endtab %}

{% tab title="pnpm" %}

```bash
pnpm add seismic-viem viem
```

{% endtab %}

{% tab title="bun" %}

```bash
bun add seismic-viem viem
```

{% endtab %}
{% endtabs %}

## Dependencies

`seismic-viem` has a single peer dependency:

| Package | Version | Role                                                    |
| ------- | ------- | ------------------------------------------------------- |
| `viem`  | 2.x     | Core Ethereum client library (transports, chains, ABIs) |

The following cryptographic libraries are bundled internally and do **not** need to be installed separately:

| Package          | Purpose                                     |
| ---------------- | ------------------------------------------- |
| `@noble/hashes`  | SHA-256, HKDF, and other hash functions     |
| `@noble/curves`  | Elliptic curve operations (secp256k1, ECDH) |
| `@noble/ciphers` | AES-GCM encryption and decryption           |

{% hint style="info" %}
The `@noble/*` packages handle all client-side cryptographic operations including ECDH key exchange with the TEE and AES-GCM calldata encryption. They are bundled as direct dependencies of seismic-viem, so you never need to install or import them yourself.
{% endhint %}

## Module Format

`seismic-viem` ships as both ESM and CJS with a single export entry point. TypeScript type declarations (`.d.ts`) are included -- no separate `@types/` package is needed.

```typescript
// ESM (recommended)
import { createShieldedWalletClient, seismicTestnet } from "seismic-viem";

// CJS
const { createShieldedWalletClient, seismicTestnet } = require("seismic-viem");
```

## Minimal Working Example

Create a new project and verify the installation:

```bash
mkdir my-seismic-app && cd my-seismic-app
npm init -y
npm install seismic-viem viem
```

Create `index.ts`:

```typescript
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { createShieldedWalletClient, seismicTestnet } from "seismic-viem";

const client = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});

const blockNumber = await client.getBlockNumber();
console.log("Connected! Block:", blockNumber);
```

Run it:

```bash
npx tsx index.ts
```

If you see a block number printed, the installation is working correctly.

## TypeScript

`seismic-viem` is written in TypeScript and ships its own type declarations. No additional configuration is required beyond a standard `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true
  }
}
```

{% hint style="info" %}
All exported types, interfaces, and function signatures are fully typed. Your editor will provide autocompletion and type checking for all seismic-viem APIs out of the box.
{% endhint %}

## Troubleshooting

### Peer Dependency Warning

If you see a peer dependency warning for `viem`, ensure you have viem 2.x installed:

```bash
npm ls viem
# Should show viem@2.x.x
```

### Node.js Version

`seismic-viem` requires Node.js 18 or newer for native `fetch` and `crypto` support:

```bash
node --version
# Should be >= v18.0.0
```

### ESM/CJS Interop

If you encounter module resolution issues, ensure your `tsconfig.json` uses `"moduleResolution": "bundler"` or `"node16"`, and that your `package.json` has `"type": "module"` if using ESM.

## See Also

- [Chains](chains.md) -- Configure network connections
- [Seismic Viem Overview](./) -- Full SDK overview and architecture
- [seismic-react](../seismic-react/) -- React hooks layer built on seismic-viem
