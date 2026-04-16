---
description: Validator staking operations on the Seismic deposit contract
icon: vault
---

# Deposit Contract

The deposit contract is the entry point for validator staking on Seismic. It accepts deposits that register validators with their node and consensus keys, and exposes read methods for the current deposit root and count. Seismic ships client extensions for both sides.

- `depositContractPublicActions` -- read-only: `getDepositRoot()`, `getDepositCount()`
- `depositContractWalletActions` -- write: `deposit()`

Both extensions are already applied by `createShieldedPublicClient` and `createShieldedWalletClient`; you can also extend a vanilla viem client with them manually.

## Import

```typescript
import {
  DEPOSIT_CONTRACT_ADDRESS,
  depositContractPublicActions,
  depositContractWalletActions,
} from "seismic-viem";
```

`DEPOSIT_CONTRACT_ADDRESS` is `0x00000000219ab540356cBB839Cbe05303d7705Fa`. Every action accepts an optional `address` to override this default.

## Extending a Client

```typescript
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  depositContractPublicActions,
  depositContractWalletActions,
  seismicTestnet,
} from "seismic-viem";
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";

const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
}).extend(depositContractPublicActions);

const walletClient = (
  await createShieldedWalletClient({
    chain: seismicTestnet,
    transport: http(),
    account: privateKeyToAccount("0x..."),
  })
).extend(depositContractWalletActions);
```

## Public Actions

### `getDepositRoot`

Returns the current deposit merkle root.

```typescript
const depositRoot = await publicClient.getDepositRoot({});
```

| Parameter | Type      | Required | Description                                          |
| --------- | --------- | -------- | ---------------------------------------------------- |
| `address` | `Address` | No       | Deposit contract address (defaults to the canonical) |

**Returns:** `Promise<Hex>` -- the SHA-256 deposit root.

### `getDepositCount`

Returns the total number of deposits accepted by the contract.

```typescript
const depositCount = await publicClient.getDepositCount({});
```

| Parameter | Type      | Required | Description                                          |
| --------- | --------- | -------- | ---------------------------------------------------- |
| `address` | `Address` | No       | Deposit contract address (defaults to the canonical) |

**Returns:** `Promise<Hex>` -- the deposit count encoded as a little-endian 64-bit number.

## Wallet Actions

### `deposit`

Registers a validator by submitting a deposit transaction.

```typescript
import { parseEther } from "viem";

const txHash = await walletClient.deposit({
  nodePubkey: "0x...", // ED25519 public key (32 bytes)
  consensusPubkey: "0x...", // BLS12-381 public key (48 bytes)
  withdrawalCredentials: "0x...", // commitment to withdrawal pubkey
  nodeSignature: "0x...", // ED25519 signature (64 bytes)
  consensusSignature: "0x...", // BLS12-381 signature (96 bytes)
  depositDataRoot: "0x...", // SHA-256 of SSZ-encoded DepositData
  value: parseEther("32"),
});
```

| Parameter               | Type      | Required | Description                                                          |
| ----------------------- | --------- | -------- | -------------------------------------------------------------------- |
| `nodePubkey`            | `Hex`     | Yes      | Validator ED25519 public key                                         |
| `consensusPubkey`       | `Hex`     | Yes      | Validator BLS12-381 consensus public key                             |
| `withdrawalCredentials` | `Hex`     | Yes      | Commitment to a public key for future withdrawals                    |
| `nodeSignature`         | `Hex`     | Yes      | ED25519 signature over the deposit data                              |
| `consensusSignature`    | `Hex`     | Yes      | BLS12-381 signature over the deposit data                            |
| `depositDataRoot`       | `Hex`     | Yes      | SHA-256 hash of the SSZ-encoded `DepositData` -- acts as a checksum  |
| `value`                 | `bigint`  | Yes      | Amount to deposit in wei                                             |
| `address`               | `Address` | No       | Deposit contract address (defaults to the canonical)                 |

**Returns:** `Promise<Hex>` -- the transaction hash.

{% hint style="info" %}
`depositDataRoot` is verified on-chain against the supplied keys and signatures. If the root does not match the SSZ-encoded deposit data, the transaction reverts. This prevents malformed deposits from being silently accepted.
{% endhint %}

## See Also

- [Shielded Public Client](shielded-public-client.md) -- base client that already includes these public actions
- [Shielded Wallet Client](shielded-wallet-client.md) -- base client that already includes these wallet actions
