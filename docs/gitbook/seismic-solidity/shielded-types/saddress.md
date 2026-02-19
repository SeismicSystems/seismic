---
description: shielded address
---

# saddress

An `saddress` variable has all `address` operations supported. As for members, it supports `call`, `delegatecall`, `staticcall`, `code`, and `codehash` _only_. You cannot have `saddress payable` or have `saddress` as a transaction signer.&#x20;

The universal casting rules and restrictions described in [Basics](./) apply.

```
saddress a = saddress(0x123);
saddress b = saddress(0x456);

// == VALID EXAMPLES
a == b  // false
b.call()

// == INVALID EXAMPLES
a.balance
payable(a)
```
