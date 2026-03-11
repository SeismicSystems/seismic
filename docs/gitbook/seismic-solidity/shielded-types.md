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

Dynamic shielded bytes mirror the standard `bytes` type. The length is stored as shielded, so observers cannot read it directly.

## Shielded Arrays

Arrays of shielded types work like standard Solidity arrays. They come in two forms:

- **Dynamic** (`suint256[]`, `sbool[]`, `saddress[]`, etc.) — the length is stored as shielded.
- **Fixed-size** (`suint256[5]`, `sbool[4]`, `saddress[3]`, etc.) — the length is a compile-time constant and publicly visible.

```solidity
suint256[] private balances;     // dynamic — shielded length
sbool[4] private flags;          // fixed — length 4 is public
```

{% hint style="warning" %}
Even with dynamic shielded arrays, an upper bound on the length may be visible to observers monitoring gas costs, since gas usage scales with array operations.
{% endhint %}

## Shielded Mappings

Mappings can have shielded values but **keys cannot be shielded types**. The standard `mapping` syntax applies.

```solidity
mapping(address => suint256) private balances;    // valid
mapping(uint256 => sbool) private flags;          // valid
mapping(address => saddress) private recipients;  // valid

mapping(saddress => uint256) private lookup;      // INVALID — shielded key
```

## Shielded Literals

{% hint style="warning" %}
Using shielded literals (e.g. `suint256(42)`) in your contract will produce a compiler warning. These literal values are embedded directly in the contract bytecode, which is publicly visible — so the initial value is leaked at deployment time.

This is fine for values meant to be public initially and then evolve through private state changes. But if the literal itself is sensitive, do not hardcode it. See [Best Practices & Gotchas](best-practices-and-gotchas.md) for more detail.
{% endhint %}
