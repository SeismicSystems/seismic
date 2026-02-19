---
description: A handle on stype unlocks all shielded computation and storage
icon: explosion
---

# Basics

## Mental model

{% hint style="info" %}
We assume familiarity with [Solidity](https://soliditylang.org/).
{% endhint %}

Developers communicate to Seismic through the `stype`. A thorough understanding of this one concept unlocks all shielded computation and storage. The `stype` consists of three elementary types:

* **`suint`** / **`sint`**: shielded integer
* **`sbool`**: shielded boolean
* **`saddress`**: shielded address

The primary difference between them and their vanilla counterparts is that they're shielded. Any operations you apply to them are carried out as expected, but the values won't be visible to external observers.

There are special considerations unique to each individual type. These are covered in the next three sections. For now, we'll develop a general understanding of `stype` that applies to all its component types.

Here's the mental model you should have for shielded contracts. Whenever a tx is broadcasted by a user, it goes through the same submission, execution, and storage phases as a tx in a regular blockchain. The only difference is that when you look at the tx at these different stages- whether it's as a calldata payload during submission, a trace during execution, or as leaves in the MPT tree during storage- any bytes that represent `stype` variables are replaced with `0x000`.

Let's step through a concrete example. We'll follow the lifecycle of a `transfer()` tx for an [`ERC20`](https://ethereum.org/en/developers/docs/standards/tokens/erc-20/) variant. This variant shields user balances and transfer amounts:

```html
mapping(address => suint256) public balanceOf;  // shielded balance

function transfer(address to, suint256 amount) public {  // shielded transfer amount
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

<figure><img src="../../.gitbook/assets/1 (1).png" alt=""><figcaption><p>Observers see 0x000 in place of stype variables during transaction submission, execution, and storage.</p></figcaption></figure>

Shielding user balances is done by changing the values of the `balanceOf` array to `suint256`. Shielding transfer amounts is done by changing the `amount` parameter in `transfer()` to `suint256`. Now we can see what happens at every stage of the tx lifecycle:

1. Submit. The tx is sitting in the mempool. You know that you're sending 12 tokens to your friend. Observers can look at the calldata and figure out that your friend is the recipient, but will see `0x000` instead of the number 12.&#x20;
2. Execute. The tx is processed by a full node, and its trace is open. You know that 12 tokens were removed from your balance and 12 were added to your friend's. Observers know that the same number that was deducted from your balance was added to your friend's, but they see `0x000` instead of the number 12.
3. Store. The effects of the tx are applied to the state tree of all full nodes. You know that your new balance goes down by 12, to 200. You know that your friend's balance went up by 12, but you only see `0x000` for what its final state is. Observers know that your new balance is down the same amount that your friend's new balance is up, but they see `0x000` for both balances.

{% hint style="info" %}
Seismic currently shields a lot more than just the bytes representing `stype` variables, so the above model is more granular than you technically need to be. However, this will soon stop being the case. You should not fit your contracts to this temporary discrepancy.
{% endhint %}

## Casting

You can cast `stype` variables to their unshielded counterparts, and vice-versa. Only explicit casting is allowed- no implicit. Note that whenever you do this, observers can look at the trace to figure out either the initial (if going from not `stype` to `stype`) or final (if going from `stype` to not `stype`) value.

```
uint256 number = 100;
suint256 sNumber = suint256(number);
```

## Restrictions

There are two restrictions in how you can use `stype` variables:

1. You can't return them in `public` or `external` functions. This also means `stype` contract variables can't be `public`, since this automatically generates a getter. If you want to return one, you'll have to cast it into its unshielded counterpart.

<pre><code><strong>/*
</strong><strong> * Throws a compiler error
</strong><strong> */
</strong><strong>suint256 public v;
</strong><strong>
</strong><strong>// ==========
</strong>
/*
 * Throws a compiler error
 */
function f() external view returns (suint256) {}
</code></pre>

2. You can't use them as constants.

```
/*
 * Throws a compiler error
 */
suint256 constant MY_CONSTANT = 42;
```
