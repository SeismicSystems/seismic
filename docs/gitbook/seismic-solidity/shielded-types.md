---
description: A handle on stype unlocks all shielded computation and storage
icon: explosion
---

# Shielded Types

## saddress - Shielded Address

An `saddress` variable has all `address` operations supported. As for members, it supports `call`, `delegatecall`, `staticcall`, `code`, and `codehash` _only_. You cannot have `saddress payable` or have `saddress` as a transaction signer.

The universal casting rules and restrictions described in [Basics](shielded-types.md) apply.

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

### sboolean - Shielded Boolean

All comparisons and operators for `sbool` function identically to `bool`. The universal casting rules and restrictions described in [Basics](shielded-types.md) apply.

We recommend reading the point on conditional execution in [Best Practices & Gotchas](best-practices-and-gotchas.md) prior to using `sbool` since it's easy to accidentally leak information with this type.

```
sbool a = sbool(true)
sbool b = sbool(false)

// == EXAMPLES
a && b  // false
!b  // true
```

### suint / sint - Shielded Unsigned Integer / Shielded Integer

All comparisons and operators for `suint` / `sint` are functionally identical to `uint` / `int`. The universal casting rules and restrictions described in [Basics](shielded-types.md) apply.

```
suint256 a = suint256(10)
suint256 b = suint256(3)

// == EXAMPLES
a > b  // true
a | b  // 11
a << 2  // 40
a % b  // 1
```
