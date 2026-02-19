---
icon: database
---

# Storage

## How Shielded Storage Works

Seismic extends the EVM storage model with **FlaggedStorage**. Every storage slot is represented as a pair:

```
(value: U256, is_private: bool)
```

The `is_private` flag determines whether a slot holds public or confidential data. This flag is set automatically by the compiler based on the types you use, and enforced at the opcode level:

- **`SSTORE` / `SLOAD`** operate on public storage slots (the standard EVM behavior).
- **`CSTORE` (`0xB1`) / `CLOAD` (`0xB0`)** operate on confidential storage slots.

When you declare a shielded variable (e.g., `suint256`), the compiler generates `CSTORE` and `CLOAD` instructions instead of `SSTORE` and `SLOAD`. This happens automatically -- you do not need to manage opcodes yourself.

### Access Control Rules

The FlaggedStorage model enforces strict separation between public and confidential data:

| Operation                 | Result                    |
| ------------------------- | ------------------------- |
| `SLOAD` on a public slot  | Returns the value         |
| `SLOAD` on a private slot | Returns `0`               |
| `CLOAD` on a private slot | Returns the value         |
| `CLOAD` on a public slot  | Returns `0`               |
| `SSTORE` to a slot        | Marks the slot as public  |
| `CSTORE` to a slot        | Marks the slot as private |

This means that if an external contract or observer uses `SLOAD` to read a shielded storage slot, they will get `0` -- not the actual value. The data is only accessible via `CLOAD`, which is what the compiler generates for shielded type access.

{% hint style="info" %}
Storage slots are not permanently fixed as public or private. The flag is set by the most recent store operation. However, mixing public and private writes to the same slot is strongly discouraged and should only be done via inline assembly with extreme care.
{% endhint %}

## Whole Slot Consumption

Shielded types consume an **entire 32-byte storage slot**, regardless of their actual size. A `suint64`, which only needs 8 bytes, still occupies a full slot.

This is a deliberate design choice. In standard Solidity, the compiler packs multiple small variables into a single slot to save gas. With shielded types, packing is not done because a storage slot must be entirely private or entirely public. Mixing shielded and unshielded data in the same slot would break the confidentiality model.

### Storage Layout Comparison

In standard Solidity, small types are packed together:

```solidity
contract RegularStorage {
    struct RegularStruct {
        uint64 a;   // Slot 0 (packed)
        uint128 b;  // Slot 0 (packed)
        uint64 c;   // Slot 0 (packed)
    }

    RegularStruct regularData;

    /*
       Storage Layout:
       - Slot 0: [a | b | c]
    */
}
```

With shielded types, each field gets its own slot:

```solidity
contract ShieldedStorage {
    struct ShieldedStruct {
        suint64 a;  // Slot 0
        suint128 b; // Slot 1
        suint64 c;  // Slot 2
    }

    ShieldedStruct shieldedData;

    /*
       Storage Layout:
       - Slot 0: [a]
       - Slot 1: [b]
       - Slot 2: [c]
    */
}
```

This means shielded contracts consume more storage slots than their unshielded equivalents. Plan your contract's storage layout accordingly.

## `saddress` Storage

An `saddress` requires **32 bytes** of storage, not the 20 bytes used by a regular `address`. This is because shielded types always consume a full 32-byte slot, and the address value is stored in its entirety within that slot.

Keep this in mind when estimating storage costs for contracts that use `saddress` extensively.

## Gas Considerations

`CLOAD` and `CSTORE` have a **constant gas cost** regardless of the value being read or written. This is a critical privacy property.

In the standard EVM, certain storage operations can have variable gas costs (e.g., writing a nonzero value to a slot that previously held zero costs more than overwriting an existing nonzero value). If `CLOAD` and `CSTORE` had similar variable costs, an observer could infer information about shielded values by analyzing gas consumption.

By making gas costs constant, Seismic prevents this class of information leakage. No matter what value is being stored or loaded, the gas cost is the same.

{% hint style="warning" %}
While `CLOAD` and `CSTORE` themselves have constant gas cost, other operations on shielded values (such as loops, conditionals, and exponentiation) can still leak information through gas. See [Best Practices & Gotchas](best-practices-and-gotchas.md) for details.
{% endhint %}

## Manual Slot Packing

If you need to pack multiple shielded values into a single slot for efficiency, you can do so using inline assembly. However, this is an advanced technique and carries significant risk.

When using inline assembly for slot packing:

- You must ensure all values packed into a single slot share the same confidentiality level.
- Incorrect packing can introduce vulnerabilities where private data is partially exposed or corrupted.
- The compiler cannot verify the correctness of your assembly-level storage operations.

```solidity
// Advanced: Manual slot packing with inline assembly
// WARNING: Only do this if you fully understand the implications.
// Ensure all packed values are shielded and belong in confidential storage.

// Example: Pack two suint128 values into a single confidential slot
function packTwo(suint128 a, suint128 b) internal {
    assembly {
        let packed := or(shl(128, a), and(b, 0xffffffffffffffffffffffffffffffff))
        // Use CSTORE to write to confidential storage
        // Slot number chosen carefully to avoid collisions
    }
}
```

{% hint style="danger" %}
Manual slot packing bypasses compiler safety checks. Use it only when absolutely necessary, and audit thoroughly. A mistake here can silently break your contract's privacy guarantees.
{% endhint %}

## Future Improvements

Compiler-level slot packing for shielded types is planned for a future release. This will allow the compiler to automatically pack multiple shielded values of compatible sizes into a single confidential slot, reducing storage costs without requiring manual assembly.

Until then, each shielded variable consumes its own full slot, and manual packing via assembly is the only alternative.
