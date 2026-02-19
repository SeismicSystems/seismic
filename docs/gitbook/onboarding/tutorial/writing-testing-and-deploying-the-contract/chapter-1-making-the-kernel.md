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

    // Constructor to initialize the kernel
    constructor(suint256 _kernel) {
        kernel = _kernel;
    }
}
```

**Add a shake function**

Next, let’s implement a function to increment the kernel. The shake function takes an **suint256** parameter, `_numShakes` which specifies the amount to increment the kernel by.&#x20;

```solidity
function shake(suint256 _numShakes) public {
    kernel += _numShakes; // Increment the kernel value using the shielded parameter.
    emit Shake(msg.sender); // Log the shake event.
}
```

**What's happening here?**

Since `shake` takes in an `stype` as one of its parameters, it is key that no information about this parameter (in this case, the number of shakes) is leaked at any time during the function call. This means that the value of `_numShakes` is known only to the function caller and is encrypted on-chain.

The function also updates a state variable (`kernel` ) and hence constitutes a state transition, which makes a call to this function a **shielded write.**&#x20;
