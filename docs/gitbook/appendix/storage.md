---
hidden: true
icon: database
---

# Storage

* **Whole Slot Consumption**: Shielded types consume an entire storage slot. This design choice ensures that a storage slot is entirely private or public, avoiding mixed storage types within a single slot.
* **Future Improvements**: We plan to support slot packing for shielded types in future updates. Until then, developers can use inline assembly to achieve slot packing manually if necessary.



**Assembly Slot Packing Example**

```
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

storage requirement for contract referring to saddress is 32 bytes, not 20 bytes
