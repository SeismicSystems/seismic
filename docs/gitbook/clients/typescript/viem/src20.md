---
description: Watch and decrypt SRC20 token events
icon: eye
---

# SRC20 Event Watching

SRC20 is Seismic's confidential token standard. Transfer and approval amounts are encrypted in the event log, so standard event watchers see only ciphertext. seismic-viem ships two client extensions that filter these events and decrypt the amounts for you:

- `src20WalletActions.watchSRC20Events` -- wallet-client action that fetches the viewing key for the connected address from the Directory contract via a signed read, then watches and decrypts events the wallet can read.
- `src20PublicActions.watchSRC20EventsWithKey` -- public-client action that takes an explicit AES viewing key. Useful for server-side monitoring, intelligence providers, or any flow without a connected wallet.

Both actions are already applied by `createShieldedWalletClient` / `createShieldedPublicClient`; you can also extend a vanilla viem client with them manually.

## How Decryption Works

1. An account registers its AES viewing key with the Directory contract.
2. When the SRC20 contract emits `Transfer` or `Approval`, the log carries:
    - `encryptKeyHash` -- `keccak256(viewingKey)`, allowing on-chain filtering
    - `encryptedAmount` -- the AES-GCM ciphertext of the amount
3. The watcher subscribes with a filter on `encryptKeyHash` so it only receives events it can decrypt.
4. For each matching log, the client decrypts `encryptedAmount` with the AES key and invokes the user callback with a `DecryptedTransferLog` / `DecryptedApprovalLog`.

## Import

```typescript
import {
  src20PublicActions,
  src20WalletActions,
} from "seismic-viem";
```

## Wallet Action: `watchSRC20Events`

Watches events for the connected wallet. The viewing key is fetched automatically from the Directory contract via a signed read.

```typescript
const unwatch = await walletClient.watchSRC20Events({
  address: "0xYourTokenAddress",
  onTransfer: (log) => {
    console.log(`Transfer ${log.from} -> ${log.to}: ${log.decryptedAmount}`);
  },
  onApproval: (log) => {
    console.log(
      `Approval ${log.owner} -> ${log.spender}: ${log.decryptedAmount}`,
    );
  },
  onError: (err) => console.error("decrypt failed:", err),
});

// stop watching
unwatch();
```

| Parameter    | Type                                    | Required | Description                                |
| ------------ | --------------------------------------- | -------- | ------------------------------------------ |
| `address`    | `Address`                               | Yes      | SRC20 token contract address               |
| `onTransfer` | `(log: DecryptedTransferLog) => void`   | No       | Called for each decrypted Transfer event   |
| `onApproval` | `(log: DecryptedApprovalLog) => void`   | No       | Called for each decrypted Approval event   |
| `onError`    | `(error: Error) => void`                | No       | Called when decryption fails               |

**Returns:** `Promise<() => void>` -- call the returned function to stop watching.

{% hint style="warning" %}
Throws if no AES key is registered in the Directory contract for the connected address. Register a viewing key first (typically during wallet setup) before watching.
{% endhint %}

## Public Action: `watchSRC20EventsWithKey`

Same filter + decryption flow, but takes an explicit viewing key instead of fetching it from the Directory.

```typescript
const viewingKey: Hex = "0x..."; // 32-byte AES key

const unwatch = await publicClient.watchSRC20EventsWithKey(viewingKey, {
  address: "0xYourTokenAddress",
  onTransfer: (log) => console.log(log),
  onApproval: (log) => console.log(log),
  onError: (err) => console.error(err),
});
```

| Parameter    | Type                            | Required | Description                              |
| ------------ | ------------------------------- | -------- | ---------------------------------------- |
| `viewingKey` | `Hex`                           | Yes      | 32-byte AES key used to decrypt amounts  |
| `params`     | `WatchSRC20EventsParams`        | Yes      | Same shape as `watchSRC20Events` above   |

**Returns:** `Promise<() => void>` -- call the returned function to stop watching.

## Log Types

```typescript
type DecryptedTransferLog = {
  from: Address;
  to: Address;
  encryptKeyHash: Hex; // keccak256 of the viewing key
  encryptedAmount: Hex; // original AES-GCM ciphertext
  decryptedAmount: bigint; // plaintext amount
  transactionHash: Hex;
  blockNumber: bigint;
};

type DecryptedApprovalLog = {
  owner: Address;
  spender: Address;
  encryptKeyHash: Hex;
  encryptedAmount: Hex;
  decryptedAmount: bigint;
  transactionHash: Hex;
  blockNumber: bigint;
};
```

## See Also

- [Shielded Public Client](shielded-public-client.md) -- base client that includes `watchSRC20EventsWithKey`
- [Shielded Wallet Client](shielded-wallet-client.md) -- base client that includes `watchSRC20Events`
- [Encrypted Events tutorial](../../../tutorials/src20/encrypted-events.md) -- end-to-end SRC20 event walkthrough
