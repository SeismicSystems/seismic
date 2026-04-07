---
description: Create and query SRC20 tokens programmatically using the @seismic/src20-sdk package
icon: code
---

# TypeScript SDK

`@seismic/src20-sdk` wraps the SRC20 Factory contract in TypeScript functions designed to work alongside `seismic-viem`.

## Installation

```bash
bun add @seismic/src20-sdk
```

## createToken

Deploys a new SRC20 token through the factory and returns the deployed address.

```typescript
import { createToken } from "@seismic/src20-sdk";
```

### Signature

```typescript
async function createToken<
  TTransport extends Transport = Transport,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account = Account,
>(
  client: ShieldedWalletClient<TTransport, TChain, TAccount>,
  params: CreateTokenParams,
): Promise<CreateTokenResult>;
```

### Parameters

| Parameter              | Type                   | Required | Description                                           |
| ---------------------- | ---------------------- | -------- | ----------------------------------------------------- |
| `client`               | `ShieldedWalletClient` | yes      | A shielded wallet client from `seismic-viem`          |
| `params.name`          | `string`               | yes      | Token name                                            |
| `params.symbol`        | `string`               | yes      | Token symbol                                          |
| `params.initialSupply` | `bigint`               | yes      | Supply in base units (e.g. `1_000_000n * 10n ** 18n`) |
| `params.decimals`      | `number`               | no       | Token decimals, defaults to `18`                      |

### Returns

```typescript
interface CreateTokenResult {
  tokenAddress: Address; // address of the deployed SRC20Token contract
  txHash: Hash; // transaction hash
}
```

### Example

```typescript
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { createShieldedWalletClient, seismicTestnet } from "seismic-viem";
import { createToken } from "@seismic/src20-sdk";

const account = privateKeyToAccount("0xYourPrivateKey");

const client = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account,
});

const { tokenAddress, txHash } = await createToken(client, {
  name: "My Private Token",
  symbol: "MPT",
  initialSupply: 1_000_000n * 10n ** 18n,
});

console.log("Token deployed at", tokenAddress);
```

---

## getTokenInfo

Reads public metadata from a deployed SRC20 token.

```typescript
import { getTokenInfo } from "@seismic/src20-sdk";
```

### Signature

```typescript
async function getTokenInfo(
  client: PublicClient,
  tokenAddress: Address,
): Promise<TokenInfo>;
```

### Returns

```typescript
interface TokenInfo {
  name: string;
  symbol: string;
  decimals: number;
  owner: Address;
  totalSupply: bigint;
}
```

### Example

```typescript
import { createPublicClient, http } from "viem";
import { seismicTestnet } from "seismic-viem";
import { getTokenInfo } from "@seismic/src20-sdk";

const client = createPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

const info = await getTokenInfo(client, "0xYourTokenAddress");
console.log(info.name, info.symbol, info.totalSupply);
```

---

## getFactoryAddress

Resolves the factory contract address for a given chain ID. Throws if the chain is not supported.

```typescript
import { getFactoryAddress } from "@seismic/src20-sdk";

const address = getFactoryAddress(5124);
// "0x87F850cbC2cFfac086F20d0d7307E12d06fA2127"
```

### FACTORY_ADDRESSES

The raw mapping of chain IDs to factory addresses:

```typescript
import { FACTORY_ADDRESSES } from "@seismic/src20-sdk";

// { 5124: "0x87F850cbC2cFfac086F20d0d7307E12d06fA2127" }
```

---

## ABIs

The package exports the full contract ABIs for use with `viem` or any other ABI-based tool:

```typescript
import { SRC20FactoryAbi, SRC20TokenAbi } from "@seismic/src20-sdk";
```

- **`SRC20FactoryAbi`** — includes `createToken`, `getTokenCount`, `tokens(uint256)`, and the `TokenCreated` event
- **`SRC20TokenAbi`** — includes `name`, `symbol`, `decimals`, `owner`, `totalSupply`, `mint`, `burn`, and all inherited SRC20 functions
