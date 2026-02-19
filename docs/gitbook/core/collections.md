---
description: Using stype variables in arrays and maps
icon: delicious
---

# Collections

All `stype` variables can be stored in Solidity collections, much like their unshielded counterparts. They behave normally (as outlined in [Basics](basics/)) when used as values in these collections. It's when they're used as _both_ the keys and values where it gets interesting. This applies to arrays and maps in particular:

```
suint256[] a;  // stype as value
function f(suint256 idx) {
    a[idx]  // stype as key
    // ...
}

// ==========

mapping(saddress => suint256) m;  // stype as key and value
function d(suint256 k) {
    m[k]
}
```

What's special here is that you can hold on to `a[idx]` and `m[k]` without observers knowing which values in the collection they refer to. You can read from these references:

```
sbool b = a[idx] < 10;
suint256 s = m[k] + 10;
```

You can write to these references:

```
a[idx] *= 3;
m[k] += a[idx];
```

Observers for any of these operations will not know which elements were read from / written to.

<figure><img src="../.gitbook/assets/figures (1).png" alt=""><figcaption><p>Using an <code>stype</code> as the key and value to a collection shields which element you're using.</p></figcaption></figure>

In the previous section, we only knew how to shield what was happening for certain elements. Now, we know how to shield which elements are being modified in the first place.

We can take the ERC20 variant discussed in the [Basics](basics/) section and extend it further to shielded balances, transfer amounts, _and now_ _recipients_.&#x20;

```
mapping(saddress => suint256) public balanceOf;  // key is now saddress

function transfer(saddress to, suint256 amount) public {  // recipient now saddress
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```
