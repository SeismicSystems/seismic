---
description: A handle on stype unlocks all shielded computation and storage
icon: explosion
---

# Shielded Types

## saddress - Shielded Address

An `saddress` variable supports `code` and `codehash` members only. Members like `call`, `delegatecall`, `staticcall`, `balance`, and `transfer` are not available — you must cast to `address` first. `saddress payable` is a valid type (see [Casting](casting.md)).

The universal casting rules and restrictions described in [Basics](shielded-types.md) apply.

```
saddress a = saddress(0x123);
saddress b = saddress(0x456);

// == VALID EXAMPLES
a == b  // false
b.code
b.codehash

// == INVALID EXAMPLES
a.balance   // must cast to address first
a.call("")  // must cast to address first
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
