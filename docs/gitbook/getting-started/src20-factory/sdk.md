---
description: API reference for the @seismic/src20-sdk package used internally by the CLI and web app
icon: code
---

# TypeScript SDK

`@seismic/src20-sdk` is the TypeScript library that powers the CLI and web interface in this repo. It wraps the SRC20 Factory contract and exposes three functions and the contract ABIs.

{% hint style="info" %}
`@seismic/src20-sdk` is an internal workspace package — it is not published to npm. It is used by the `packages/cli` and `packages/web` packages within this monorepo. This page documents its API surface as a reference.
{% endhint %}

---

## createToken

Deploys a new SRC20 token through the factory and returns the deployed address.

### Signature

```typescript
async function createToken(
  client: ShieldedWalletClient,
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

---

## getTokenInfo

Reads public metadata from a deployed SRC20 token using a standard public client.

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
  totalSupply: bigint; // in base units
}
```

---

## getFactoryAddress

Resolves the factory contract address for a given chain ID. Throws if the chain is not supported.

### Signature

```typescript
function getFactoryAddress(chainId: number): Address;
```

### FACTORY_ADDRESSES

The raw mapping of chain IDs to factory addresses:

```typescript
const FACTORY_ADDRESSES: Record<number, Address> = {
  5124: "0x87F850cbC2cFfac086F20d0d7307E12d06fA2127",
};
```

---

## ABIs

The package exports the full contract ABIs:

- **`SRC20FactoryAbi`** — `createToken`, `getTokenCount`, `tokens(uint256)`, `TokenCreated` event
- **`SRC20TokenAbi`** — `name`, `symbol`, `decimals`, `owner`, `totalSupply`, `mint`, `burn`, and all inherited SRC20 functions (`transfer`, `transferFrom`, `approve`, `balance`, `allowance`, `balanceOfSigned`, `permit`)
