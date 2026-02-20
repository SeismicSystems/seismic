---
icon: hammer-crash
---

# Chapter 2: Making the Shell and revealing the Kernel

In this chapter, you’ll build the **shell**, the protective layer that hides the kernel. You’ll initialize the shell’s strength and implement a `hit` function to decrement it. Additionally, you’ll add a `look()` function with a `requiredCracked` modifier to ensure the kernel can only be viewed once the shell is fully broken. _Estimated Time: \~10 minutes._

### Defining the shell

The shell determines the Walnut’s resilience. It has an integer strength (`shellStrength`), which represents how many hits it can withstand before cracking. Let’s define the shell and initialize it in the constructor:

```solidity
    uint256 shellStrength; // The strength of the Walnut's shell.

    constructor(uint256 _shellStrength, suint256 _kernel) {
        shellStrength = _shellStrength; // Set the initial shell strength.
        kernel = _kernel; // Initialize the kernel.
    }
```

### Adding the hit function

Each time the Walnut is hit, the shell strength decreases, simulating damage to the protective shell. This is crucial for revealing the kernel, **as the shell must be fully broken for the kernel to be accessed:**

<pre class="language-solidity"><code class="lang-solidity"><strong>    // Event to log hits
</strong>    event Hit(address indexed hitter, uint256 remainingShellStrength);
    
    // Function to hit the walnut shell
    function hit() public {
        shellStrength--; // Decrease the shell strength.
        emit Hit(msg.sender, shellStrength); // Log the hit event.
    }

    // Modifier to ensure the shell is not cracked.
    modifier requireIntact() {
        require(shellStrength > 0, "SHELL_ALREADY_CRACKED");
        _;
    }
</code></pre>

### What's happening here?

* **The `requireIntact`** modifier: Ensures that the function cannot be called if the Walnut’s shell is already broken (`shellStrength == 0`). This prevents unnecessary calls after the shell is fully cracked. We can now also add this modifier to the shake function in order to restrict `shake` being called even after the shell is broken:

```solidity
    function shake(suint256 _numShakes) public requireIntact {
        kernel += _numShakes; // Increment the kernel value using the shielded parameter.
        emit Shake(msg.sender); // Log the shake event.
    }
```

* **Decrementing the shell**: Each call to `hit`decreases the shell’s strength (`shellStrength`) by one.
* **Logging the action**: The `Hit` event records the hitter’s address `(msg.sender)` and the remaining shell strength.

### Example call:

Here’s how calling the hit function works in practice:

• **Initial State**: The shell strength is set to 5.

• **First Hit**: A player calls hit(). The shell strength decreases to 4.

• **Subsequent Hits**: Each additional hit reduces the shell strength by 1 until it reaches 0

### Revealing the Kernel

Now that we have implemented the shell’s durability and the ability to break it using the hit function, we can introduce a new condition: the kernel should only be revealed once the shell is fully cracked.

Currently, there is no way to access the kernel’s value. However, now that we have a shell with a decreasing strength, we can apply a condition that restricts when the kernel can be seen. Specifically:

• The kernel **should remain hidden** while the **shell is intact**.

• The kernel **can only be revealed** once the shell’s strength reaches **zero, i.e. when it is cracked**.

To enforce this, we will create a function called `look()` , which will return the kernel’s value, but only if the Walnut has been fully cracked.

Here’s how we define `look()` with a `requireCracked` modifier:

```solidity
    // Function to reveal the kernel if the shell is fully cracked.
    function look() public view requireCracked returns (uint256) {
        return uint256(kernel); // Reveal the kernel as a standard uint256.
    }
    
    // Modifier to ensure the shell is fully cracked before revealing the kernel.
    modifier requireCracked() {
        require(shellStrength == 0, "SHELL_INTACT"); // Ensure the shell is broken before revealing the kernel.
        _;
    }
```

**What's happening here?**

* **Restricting Access with a Condition**: The `requireCracked` modifier ensures that look() can only be called if `shellStrength == 0`, meaning the Walnut has been fully cracked.
* **Revealing the Kernel**: Once the condition is met, `look()` returns the unshielded value of the kernel.
* **Preventing Premature Access**: If look() is called before the shell is broken, the function will revert with the error `"SHELL_INTACT"`.

### Updated contract with hit, shake and look

<pre class="language-solidity"><code class="lang-solidity">// SPDX-License-Identifier: MIT License
pragma solidity ^0.8.13;

contract Walnut {
    uint256 shellStrength; // The strength of the Walnut's shell.
    suint256 kernel; // The hidden kernel (number inside the Walnut).

    // Events
    event Hit(address indexed hitter, uint256 remainingShellStrength); // Logs when the Walnut is hit.
    event Shake(address indexed shaker); // Logs when the Walnut is shaken.

    // Constructor to initialize the shell and kernel.
    constructor(uint256 _shellStrength, suint256 _kernel) {
        shellStrength = _shellStrength; // Set the initial shell strength.
        kernel = _kernel; // Initialize the kernel.
    }

    // Function to hit the Walnut and reduce its shell strength.
    function hit() public requireIntact {
        shellStrength--; // Decrease the shell strength.
        emit Hit(msg.sender, shellStrength); // Log the hit action.
    }

    // Function to shake the Walnut and increment the kernel.
    function shake(suint256 _numShakes) public requireIntact {
        kernel += _numShakes; // Increment the kernel by the given number of shakes.
        emit Shake(msg.sender); // Log the shake action.
    }
    
    // Function to reveal the kernel if the shell is fully cracked.
    function look() public view requireCracked returns (uint256) {
        return uint256(kernel); // Reveal the kernel as a standard uint256.
    }
    
    // Modifier to ensure the shell is fully cracked before revealing the kernel.
<strong>    modifier requireCracked() {
</strong>        require(shellStrength == 0, "SHELL_INTACT"); // Ensure the shell is broken before revealing the kernel.
        _;
    }
    
    // Modifier to ensure the shell is not cracked.
    modifier requireIntact() {
        require(shellStrength > 0, "SHELL_ALREADY_CRACKED");
        _;
    }
    
}
</code></pre>
