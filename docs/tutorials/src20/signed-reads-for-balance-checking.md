---
description: "Let users check their own balance without exposing it to others"
icon: signature
---

# Signed Reads for Balance Checking

The SRC20 contract stores balances as `suint256`, which means there is no public getter. This chapter shows how to let users query their own balance securely using signed reads. _Estimated time: ~15 minutes._

## The problem

In a standard ERC20, `balanceOf` is a `public` mapping. Anyone can call `balanceOf(address)` and see any account's balance. In the SRC20, two things prevent this:

1. **Shielded types cannot be `public`.** The `suint256` value type means the automatic getter would try to return a shielded type from an external function, which the compiler rejects.

2. **Vanilla `eth_call` has no sender identity.** On Seismic, the `from` field is zeroed out for unsigned `eth_call` requests. This means a contract cannot verify `msg.sender` in a view function called via a normal `eth_call` -- `msg.sender` would just be `address(0)`. Without sender verification, anyone could impersonate any address and read their balance.

So you need two things: a way for the contract to verify who is asking, and a way to return the value only to that person.

## Signed reads

A **signed read** solves both problems. It is a Seismic transaction (type `0x4A`) sent to the `eth_call` RPC endpoint instead of `eth_sendRawTransaction`. Because it is a valid signed transaction:

- The `from` field is cryptographically verified, so the contract can trust `msg.sender`.
- The response is encrypted to the sender's encryption public key (included in the Seismic transaction's elements). Even if someone intercepts the response, they cannot decrypt it.
- The `signed_read` flag in SeismicElements is set to `true`, which prevents anyone from replaying this payload as a write transaction.

From the contract's perspective, a signed read looks like a normal view function call. The difference is entirely at the transport layer.

## Contract implementation

Add a `balanceOf` function that requires the caller to be the account owner:

```solidity
function balanceOf(address account) external view returns (uint256) {
    require(msg.sender == account, "Only owner can view balance");
    return uint256(balanceOf[account]);
}
```

{% hint style="info" %}
Note the naming here. The mapping `balanceOf` is internal (no `public` modifier), so there is no collision with the function name. The function acts as the explicit getter that replaces the auto-generated one.
{% endhint %}

This function does three things:

1. **Checks `msg.sender`** -- Only the account owner can query their own balance. With a vanilla `eth_call`, `msg.sender` would be `address(0)` and this check would always fail. With a signed read, `msg.sender` is the actual caller.

2. **Casts to `uint256`** -- The shielded `suint256` value is cast to a regular `uint256` for the return value. Shielded types cannot be returned from external functions.

3. **Returns the balance** -- The return value is encrypted to the caller's key by the Seismic node before being sent back. Even though the function returns a `uint256`, the response payload is encrypted.

### Allowance checking

The same pattern applies to allowances:

```solidity
function allowanceOf(address owner, address spender) external view returns (uint256) {
    require(
        msg.sender == owner || msg.sender == spender,
        "Not authorized"
    );
    return uint256(allowance[owner][spender]);
}
```

Here, either the owner or the spender can check the allowance. Both parties have a legitimate reason to know the value.

## Client-side code

On the client side, signed reads are handled transparently by `seismic-viem`. When you use a `ShieldedWalletClient` or a `ShieldedContract`, read calls are automatically sent as signed reads.

### Using a shielded contract instance

```typescript
import { createShieldedWalletClient, getShieldedContract } from "seismic-viem";
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";

// Create a shielded wallet client
const walletClient = await createShieldedWalletClient({
  chain: seismicDevnet,
  transport: http("https://rpc-devnet.seismic.systems"),
  account: privateKeyToAccount(PRIVATE_KEY),
});

// Get a contract instance
const token = getShieldedContract({
  abi: src20Abi,
  address: SRC20_ADDRESS,
  client: walletClient,
});

// Read balance -- this is automatically a signed read
const balance = await token.read.balanceOf([walletClient.account.address]);
console.log("My balance:", balance);
```

The `token.read.balanceOf()` call is sent as a signed read under the hood. The wallet client signs the request, the node verifies the signature, executes the view function with the correct `msg.sender`, and encrypts the response to the caller's key.

### Using the wallet client directly

You can also call `readContract` on the wallet client:

```typescript
const balance = await walletClient.readContract({
  address: SRC20_ADDRESS,
  abi: src20Abi,
  functionName: "balanceOf",
  args: [walletClient.account.address],
});
```

Both approaches produce the same signed read.

## Security model

The signed read has several layers of protection:

| Property                  | How it works                                                                                                                                                     |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Sender authentication** | The transaction is signed by the user's private key. The node verifies the signature before executing.                                                           |
| **Response encryption**   | The response is encrypted to the user's encryption public key, included in the SeismicElements of the transaction.                                               |
| **Replay protection**     | The `signed_read` flag is set to `true`. If someone intercepts the signed read payload and submits it to `eth_sendRawTransaction`, it is rejected.               |
| **No state changes**      | A signed read is sent to `eth_call`, which does not modify state. Even if the replay protection were bypassed, the function is `view` and cannot alter balances. |

The result is that only the account owner can view their balance, and the balance is never exposed in plaintext outside the TEE.

## A note on privacy tradeoffs

With this implementation, each user can only see their own balance. They cannot see anyone else's balance. This is the strictest privacy model. In the next chapter, [Intelligence Contracts](intelligence-contracts.md), we will add authorized roles (such as compliance officers) who can view specific balances when required -- without compromising privacy for everyone else.
