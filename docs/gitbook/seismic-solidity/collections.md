---
description: Using stype variables in arrays and maps
icon: delicious
---

# Collections

## Shielded Arrays

Arrays of shielded types work like standard Solidity arrays. Keys (indices) must be non-shielded — using a shielded type as an array index is a compiler error.

They come in two forms:

- **Dynamic** (`suint256[]`, `sbool[]`, `saddress[]`, etc.) — the length is stored as shielded.
- **Fixed-size** (`suint256[5]`, `sbool[4]`, `saddress[3]`, etc.) — the length is a compile-time constant and publicly visible.

```solidity
suint256[] private balances;     // dynamic — shielded length
sbool[4] private flags;          // fixed — length 4 is public

function example(uint256 i) public {
    balances[i] = suint256(100);   // valid — uint256 index
    flags[0] = sbool(true);        // valid — literal index
}
```

{% hint style="warning" %}
Even with dynamic shielded arrays, an upper bound on the length may be visible to observers monitoring gas costs, since gas usage scales with array operations.
{% endhint %}

## Shielded Mappings

Mappings can have shielded values but **keys cannot be shielded types**. Using a shielded type as a mapping key is a compiler error. The standard `mapping` syntax applies.

```solidity
mapping(address => suint256) private balances;    // valid
mapping(uint256 => sbool) private flags;          // valid
mapping(address => saddress) private recipients;  // valid

mapping(saddress => uint256) private lookup;      // INVALID — shielded key
```
