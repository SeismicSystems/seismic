---
description: Select native funds or a registered token for each Seismic transaction
icon: gas-pump
---

# Gas Payment

`gasPayment` is an optional **top-level request option**. Omission resolves to Auto before signing; the signed Seismic wire field is always present.

```typescript
import type { GasPayment } from 'seismic-viem';
import type { Address } from 'viem';

const gasTokenAddress: Address = '0xYourGasTokenAddress';
const payment: GasPayment = { type: 'token', token: gasTokenAddress };

await client.sendShieldedTransaction({
  to: '0xYourContractAddress',
  data: calldata,
  gasPayment: payment,
});

await token.swrite.transfer(
  ['0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266', 1000n],
  { gasPayment: payment },
);
```

Replace placeholder addresses with valid 20-byte addresses.

| Choice | Meaning |
| --- | --- |
| Omitted or `{ type: 'auto' }` | Native funds first, then eligible active registry tokens in insertion order |
| `{ type: 'native' }` | Native funds only; no token fallback |
| `{ type: 'token', token: address }` | Exactly this registered token; no fallback |

Token addresses must be nonzero. Auto/Native cannot carry a token, and unknown tags or extra properties are rejected. The fee token need not be the contract being called; for proxies, select the registered proxy address, not its implementation.

## Supported paths

The option is forwarded by `sendShieldedTransaction`, `shieldedWriteContract`, `shieldedWriteContractDebug`, wallet `swriteContract`/`dwriteContract`, contract `.swrite`/`.dwrite`, `signedCall`, `signedReadContract`, and contract `.sread`. Separately signed gas-estimation transactions use the same choice. Debug transaction views include the resolved choice.

Smart `.write`/`.read` routing remains based on the ABI. If it chooses the transparent path, only Auto is accepted: standard Ethereum envelopes cannot authenticate Native or Token selection. Use `.swrite`/`.sread` or the corresponding explicit shielded wallet helpers when you need a non-Auto selector. Non-Auto requests are rejected rather than silently dropped or routed differently.

React's `useShieldedWriteContract` and `useSignedReadContract` accept the same `gasPayment` option.

## Automatic gas limits

When `gas` is omitted on a shielded write, the SDK signs a separate read-only estimation twin, then sets the final transaction's gas limit to `max(execution estimate, encrypted-input admission minimum)`. The minimum is calculated from the **final write ciphertext**, not the separately encrypted estimation request.

The current Seismic pool uses Prague intrinsic gas and the calldata gas floor, including applicable creation/initcode and authorization-list charges; Seismic ignores access-list charges. This bound is conservative on pre-Prague nodes. It prevents cheap plaintext executions from being rejected because their encrypted input has a higher admission minimum.

An explicit `gas` value is preserved, even if the node will reject it as too low. Raising the automatically selected limit can increase the upfront fee reserve, but does not itself charge the full limit. Signed calls and transparent transaction paths are unchanged.

## Public signed metadata

The selector is public, authenticated metadata, **not encrypted**. It does not belong in `securityParams`, `SeismicElements`, or AEAD/AAD. Selecting a different fee asset does not change the native-wei meaning of `value`, `gasPrice`, or receipt gas fields.

The registry's token decimals determine fee conversion. At the fixed protocol rate, token fees are rounded up: `ceil(native_wei_fee / 10^(18 - decimals))`. Registration does not mint tokens or ensure that the sender has an eligible balance.

Signed calls and gas estimates carry this field but do not charge canonical state or consume a transaction nonce.

## Node compatibility

This SDK format requires a node that supports the mandatory selector. Upgrading the SDK allows existing application calls to omit the option and get Auto; it does **not** make old, already-signed raw/EIP-712 messages valid on new-format nodes. Standard Ethereum transaction encoding is unchanged. No custom raw decoder or old-wire compatibility mode is added.
