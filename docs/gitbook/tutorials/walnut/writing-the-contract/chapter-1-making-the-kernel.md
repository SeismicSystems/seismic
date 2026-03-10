---
icon: acorn
---

# Chapter 1: Making the Kernel

In this chapter, you’ll learn to create and initialize the kernel, a hidden value inside the Walnut, and increment it by implementing a shake function. _Estimated time: \~10 minutes._

### Defining the kernel

The **kernel** is the hidden number inside the Walnut. Using Seismic’s **`suint256`** type, the kernel is shielded on-chain. Open up `packages/contracts/Walnut.sol` and define the kernel as a state variable and initialize it in the constructor:

```solidity
// SPDX-License-Identifier: MIT License
pragma solidity ^0.8.13;

contract Walnut {
    suint256 kernel; // The shielded kernel (number inside the Walnut)

    event Shake(address indexed player);

    // Constructor to initialize the kernel
    // Note: use uint256 here, not suint256 — constructor calldata is not
    // encrypted (CREATE/CREATE2 does not use Seismic transactions), so
    // shielded constructor parameters would leak their values.
    constructor(uint256 _kernel) {
        kernel = suint256(_kernel);
    }
}
```

**Add a shake function**

Next, let’s implement a function to increment the kernel. The shake function takes an **suint256** parameter, `_numShakes` which specifies the amount to increment the kernel by.

```solidity
function shake(suint256 _numShakes) public {
    kernel += _numShakes; // Increment the kernel value using the shielded parameter.
    emit Shake(msg.sender); // Log the shake event.
}
```

**What's happening here?**

Since `shake` takes a shielded type as a parameter, you should use a Seismic transaction to keep the input private. The value of `_numShakes` is known only to the function caller and is shielded on-chain.

The function also updates a shielded state variable (`kernel`), so you should use a Seismic transaction (a **shielded write**) when calling it.
