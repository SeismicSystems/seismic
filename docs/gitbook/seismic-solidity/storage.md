---
icon: database
metaLinks:
  alternates:
    - https://app.gitbook.com/s/hkB2uNxma1rxIgBfHgAT/appendix/storage
---

# Storage

## How Shielded Storage Works

Seismic extends the EVM storage model with **FlaggedStorage**. Every storage slot is represented as a pair:

```
(value: U256, is_private: bool)
```

The `is_private` flag determines whether a slot holds public or confidential data. This flag is set automatically by the compiler based on the types you use, and enforced at the opcode level:

* **`SSTORE` / `SLOAD`** operate on public storage slots (the standard EVM behavior).
* **`CSTORE` (`0xB1`) / `CLOAD` (`0xB0`)** operate on confidential storage slots.

When you declare a shielded variable (e.g., `suint256`), the compiler generates `CSTORE` and `CLOAD` instructions instead of `SSTORE` and `SLOAD`. This happens automatically -- you do not need to manage opcodes yourself.

### Access Control Rules

The FlaggedStorage model enforces strict separation between public and confidential data:

| Operation                          | Result                                  |
| ---------------------------------- | --------------------------------------- |
| `SLOAD` on a public slot           | Returns the value                       |
| `SLOAD` on a private slot          | Reverts                                 |
| `CLOAD` on a private slot          | Returns the value                       |
| `CLOAD` on a public slot           | Returns the value                       |
| `SSTORE` to a public slot          | Marks the slot as public                |
| `SSTORE` to a private slot         | Reverts                                 |
| `CSTORE` to a private slot         | Marks the slot as private               |
| `CSTORE` to a zero-value public slot | Claims the slot as private            |
| `CSTORE` to a non-zero public slot | Reverts                                 |

This means that if an external contract or observer uses `SLOAD` to read a shielded storage slot, the operation will revert. `CLOAD` can access both private and public slots — the compiler generates `CLOAD` for all shielded type access.

## Whole Slot Consumption

Shielded types consume an **entire 32-byte storage slot**, regardless of their actual size. A `suint64`, which only needs 8 bytes, still occupies a full slot.

This is a deliberate design choice. In standard Solidity, the compiler packs multiple small variables into a single slot to save gas. With shielded types, packing is not done for two reasons: a storage slot must be entirely private or entirely public (mixing would break the confidentiality model), and packing would leak the size of the shielded value (see [Gas Considerations](#gas-considerations) below).

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

## Gas Considerations

`CLOAD` and `CSTORE` have a **constant gas cost** regardless of the value being read or written. This is a critical privacy property.

In the standard EVM, certain storage operations can have variable gas costs (e.g., writing a nonzero value to a slot that previously held zero costs more than overwriting an existing nonzero value). If `CLOAD` and `CSTORE` had similar variable costs, an observer could infer information about shielded values by analyzing gas consumption.

By making gas costs constant, Seismic prevents this class of information leakage. No matter what value is being stored or loaded, the gas cost is the same.

{% hint style="warning" %}
While `CLOAD` and `CSTORE` themselves have constant gas cost, other operations on shielded values (such as loops, conditionals, and exponentiation) can still leak information through gas. See [Be Mindful of Gas-Based Information Leakage](best-practices-and-gotchas.md#2.-be-mindful-of-gas-based-information-leakage) for details.
{% endhint %}

## Manual Slot Packing

If you need to pack multiple shielded values into a single slot for efficiency, you can do so using inline assembly. However, this is an advanced technique and carries significant risk.

When using inline assembly for slot packing:

* You must ensure all values packed into a single slot share the same confidentiality level.
* Incorrect packing can introduce vulnerabilities where private data is partially exposed or corrupted.
* The compiler cannot verify the correctness of your assembly-level storage operations.

```solidity
contract ManualSlotPacking {
    // Use a deterministic slot derived from a namespace string to avoid collisions.
    // keccak256("ManualSlotPacking.packed") = a fixed slot number.

    function _packedSlot() internal pure returns (uint256 s) {
        assembly {
            s := keccak256(0, 0)  // placeholder, we use a constant below
        }
        // Use a constant derived from a namespace to avoid storage collisions.
        s = uint256(keccak256("ManualSlotPacking.packed"));
    }

    function packTwo(suint128 a, suint128 b) public {
        uint256 slot = _packedSlot();
        assembly {
            let packed := or(shl(128, a), and(b, 0xffffffffffffffffffffffffffffffff))
            cstore(slot, packed)
        }
    }

    function unpackTwo() public view returns (uint128, uint128) {
        uint256 slot = _packedSlot();
        uint256 packed;
        assembly {
            packed := cload(slot)
        }
        uint128 a = uint128(packed >> 128);
        uint128 b = uint128(packed);
        return (a, b);
    }
}
```

{% hint style="danger" %}
Manual slot packing bypasses compiler safety checks. Use it only when absolutely necessary, and audit thoroughly. A mistake here can silently break your contract's privacy guarantees.
{% endhint %}

## Future Improvements

Compiler-level slot packing for shielded types is planned for a future release. This will allow the compiler to automatically pack multiple shielded values of compatible sizes into a single confidential slot, reducing storage costs without requiring manual assembly.

Until then, each shielded variable consumes its own full slot, and manual packing via assembly is the only alternative.
