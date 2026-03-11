---
description: A handle on stype unlocks all shielded computation and storage
icon: explosion
---

# Shielded Types

Operations on shielded types return shielded types. For example, comparing two `suint256` values produces an `sbool`, not a `bool`. Arithmetic on `sint256` returns `sint256`, and so on.

## Shielded Integers

All comparisons and operators for shielded integers are functionally identical to their unshielded counterparts.

### suint - Shielded Unsigned Integer

```
suint256 a = suint256(10);
suint256 b = suint256(3);

// == EXAMPLES
a > b   // sbool(true)
a | b   // suint256(11)
a << 2  // suint256(40)
a % b   // suint256(1)
```

### sint - Shielded Signed Integer

```
sint256 a = sint256(-10);
sint256 b = sint256(3);

// == EXAMPLES
a < b   // sbool(true)
a + b   // sint256(-7)
a * b   // sint256(-30)
```

## sbool - Shielded Boolean

All comparisons and operators for `sbool` function identically to `bool`.

We recommend reading the point on [conditional execution](best-practices-and-gotchas.md#1.-conditional-execution) prior to using `sbool` since it's easy to accidentally leak information with this type.

```
sbool a = sbool(true);
sbool b = sbool(false);

// == EXAMPLES
a && b  // sbool(false)
!b      // sbool(true)
```

## saddress - Shielded Address

An `saddress` variable supports `code` and `codehash` members only. Members like `call`, `delegatecall`, `staticcall`, `balance`, and `transfer` are not available — you must cast to `address` first.

```
saddress a = saddress(0x123);
saddress b = saddress(0x456);

// == VALID EXAMPLES
a == b  // sbool(false)
b.code
b.codehash

// == INVALID EXAMPLES
a.balance   // must cast to address first
a.call("")  // must cast to address first
```

## sbytes - Shielded Bytes

### Fixed-size: sbytes1 through sbytes32

Fixed-size shielded bytes mirror the standard `bytes1`–`bytes32` types.

```
sbytes32 a = sbytes32(0xabcd);
sbytes1 b = sbytes1(0xff);
```

### Dynamic: sbytes

Dynamic shielded bytes mirror the standard `bytes` type. The length is stored as shielded — like [dynamic shielded arrays](collections.md#shielded-arrays), observers cannot read it directly but may infer an upper bound from gas costs.

## Shielded Literals

{% hint style="warning" %}
Using shielded literals (e.g. `suint256(42)`) in your contract will produce a compiler warning. These literal values are embedded directly in the contract bytecode, which is publicly visible — so the initial value is leaked at deployment time.

This is fine for values meant to be public initially and then evolve through private state changes. But if the literal itself is sensitive, do not hardcode it. See [Best Practices & Gotchas](best-practices-and-gotchas.md) for more detail.
{% endhint %}
