---
icon: hashtag
---

# Shielded Literals

The `s` suffix lets you write shielded integer constants directly, without explicit casting. The compiler infers the concrete shielded type (`suint8`, `suint256`, `sint128`, etc.) from context.

```solidity
suint256 x = 42s;           // inferred as suint256
suint8 small = 7s;           // inferred as suint8
sint256 neg = -1s;           // inferred as sint256
```

This is equivalent to:

```solidity
suint256 x = suint256(42);
suint8 small = suint8(7);
sint256 neg = sint256(-1);
```

Both forms are valid. The `s` suffix is syntactic sugar — it produces the same bytecode.

## Supported formats

The `s` suffix works with all numeric literal forms:

```solidity
suint256 a = 1_000s;         // underscores
suint256 b = 0xDEADs;        // hex
suint256 c = 1e5s;           // scientific notation
sint256  d = -42s;            // unary minus
```

## Type inference

The shielded type is inferred from the assignment target or surrounding expression:

```solidity
suint8  a = 255s;            // suint8
suint32 b = 1_000s;          // suint32
sint128 c = -1s;             // sint128

suint256 x = 10s;
suint256 y = x + 5s;         // 5s inferred as suint256 from context
```

## Rules

**No mixed arithmetic.** You cannot mix shielded and non-shielded literals in the same expression:

```solidity
suint256 x = 1s + 1;         // Error — mixed shielded/non-shielded
suint256 x = 1s + 1s;        // OK
```

**No implicit conversion to non-shielded types.** A shielded literal cannot be assigned to a regular type:

```solidity
uint256 x = 42s;             // Error — cannot assign shielded to uint256
```

**No `immutable` or `constant`.** Shielded literals follow the same restriction as all shielded types — they cannot be used in `immutable` or `constant` declarations. See [Footguns](footguns.md#immutable-and-constant-shielded-variables).

## Compiler warning

All shielded literals emit **warning 9660**. This is intentional — literal values are embedded in the contract bytecode, which is publicly visible. The warning reminds you that the value is leaked at deployment time.

If the literal value is sensitive, do not hardcode it. Introduce it via encrypted calldata instead. See [Footguns](footguns.md#literals) for more detail.
