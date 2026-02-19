---
icon: people-line
---

# Chapter 3: Reset Mechanism, Rounds, and a more conditional Kernel Reveal

In this chapter, we’ll implement a reset mechanism that allows the Walnut to be reused in multiple rounds, ensuring each game session starts fresh. We’ll also track contributors per round so that only players who participated in cracking the Walnut can call `look()`. By the end, we’ll have a fully functional round-based walnut game where the kernel remains shielded until conditions are met! _Estimated time: \~15 minutes._

### The need for a Reset mechanism

Right now, once the Walnut is cracked, there’s no way to reset it. If a game session were to continue, we’d have no way to start fresh—the shell would remain at 0, and the kernel would be permanently revealed.

To solve this, we need to introduce:

✅ A `reset`  function that restores the Walnut to its original state.

✅ Round tracking, so each reset creates a new round.

### The need for a contributor check

While the reset mechanism and round tracking allow us to restart the Walnut for continuous gameplay, they still don’t address **who** should be allowed to call the `look()` function.

Right now, any player can call `look()` once the shell is cracked, even if they didn’t participate in hitting it during the current round. This creates the following issues:

* **Fairness**: Players who didn’t contribute should not be able to reap the benefits of seeing the kernel.
* &#x20;**Incentivizing Contribution**: The game needs to encourage active participation by ensuring that only those who helped crack the Walnut in a specific round are rewarded with access to the kernel.

The solution to this is implementing a conditional check on `look()` which allows only those players who **contributed in hitting the shell for a particular round (i.e., players whose hit count is >0 for that round)** to view the kernel after the walnut is cracked.

### Implementing the Reset Mechanism

The reset mechanism allows the Walnut to be reused for multiple rounds, with each round starting fresh. It restores the Walnut’s shell and kernel to their original states and increments the round counter to mark the beginning of a new round.

Here’s how we can implement the reset function:

```solidity
    // The current round number.
    uint256 round; 
    
    // Event to log resets.
    event Reset(uint256 indexed newRound, uint256 shellStrength);
    
    function reset() public requireCracked {
        shellStrength = initialShellStrength; // Restore the shell strength.
        kernel = initialKernel; // Reset the kernel to its original value.
        round++; // Increment the round counter.
        emit Reset(round, shellStrength); // Log the reset action.
    }
```

**What’s Happening Here?**

* **Condition for Reset (`requireCracked`):** The reset function can only be called once the Walnut’s shell is cracked, enforced by the `requireCracked` modifier.
* **Restoring Initial State**: The shell strength and kernel are reset to their original values (`initialShellStrength` and `initialKernel`), ensuring the Walnut starts afresh for the next round.&#x20;
* **Round Tracking**: The `round` counter increments each time the Walnut is reset, allowing us to distinguish between rounds.

### Modifying hit() to track contributions

To enforce fair access to the kernel, we’ll track the number of hits each player contributes in a given round. This is achieved using the `hitsPerRound` mapping:

```solidity
    // Mapping to track contributions: hitsPerRound[round][player] → number of hits.
    mapping(uint256 => mapping(address => uint256)) hitsPerRound;
```

Every time a player calls the `hit()` function, we update their contribution in the current round:

```solidity
    function hit() public requireIntact {
        shellStrength--; // Decrease the shell strength.
        hitsPerRound[round][msg.sender]++; // Record the player's contribution for the current round.
        emit Hit(round, msg.sender, shellStrength); // Log the hit event.
    }
```

**What’s Happening Here?**

* **Tracking Contributions**: The `hitsPerRound` mapping records each player’s hits in the current round. This ensures we can verify who participated when the Walnut was cracked.
* **Replayable Rounds**:  Because contributions are tracked by round, the game can fairly reset and start fresh without losing player data from previous rounds.

### Restricting look() with a contributor check

To ensure only contributors can reveal the kernel, we’ll use a modifier called `onlyContributor`:

```solidity
     modifier onlyContributor() {
        require(hitsPerRound[round][msg.sender] > 0, "NOT_A_CONTRIBUTOR"); // Check if the caller contributed in the current round.
        _;
    }
```

We’ll then apply this modifier to the `look()` function:

```solidity
    // Look at the kernel if the shell is cracked and the caller contributed.
    function look() public view requireCracked onlyContributor returns (uint256) {
        return uint256(kernel); // Return the kernel value.
    }
```

**Congratulations!** You made it through to writing the entire shielded smart contract for a multiplayer, multi-round, walnut app!&#x20;

**Final Walnut contract**

```solidity
// SPDX-License-Identifier: MIT License
pragma solidity ^0.8.13;

contract Walnut {
    uint256 initialShellStrength; // The initial shell strength for resets.
    uint256 shellStrength; // The current shell strength.
    uint256 round; // The current round number.

    suint256 initialKernel; // The initial hidden kernel value for resets.
    suint256 kernel; // The current hidden kernel value.

    // Tracks the number of hits per player per round.
    mapping(uint256 => mapping(address => uint256)) hitsPerRound;

    // Events to log hits, shakes, and resets.

    // Event to log hits.
    event Hit(uint256 indexed round, address indexed hitter, uint256 remaining);
    // Event to log shakes.
    event Shake(uint256 indexed round, address indexed shaker);
    // Event to log resets.
    event Reset(uint256 indexed newRound, uint256 shellStrength);

    constructor(uint256 _shellStrength, suint256 _kernel) {
        initialShellStrength = _shellStrength; // Set the initial shell strength.
        shellStrength = _shellStrength; // Initialize the shell strength.

        initialKernel = _kernel; // Set the initial kernel value.
        kernel = _kernel; // Initialize the kernel value.

        round = 1; // Start with the first round.
    }

    // Get the current shell strength.
    function getShellStrength() public view returns (uint256) {
        return shellStrength;
    }

    // Hit the Walnut to reduce its shell strength.
    function hit() public requireIntact {
        shellStrength--; // Decrease the shell strength.
        hitsPerRound[round][msg.sender]++; // Record the player's hit for the current round.
        emit Hit(round, msg.sender, shellStrength); // Log the hit.
    }

    // Shake the Walnut to increase the kernel value.
    function shake(suint256 _numShakes) public requireIntact {
        kernel += _numShakes; // Increment the kernel value.
        emit Shake(round, msg.sender); // Log the shake.
    }

    // Reset the Walnut for a new round.
    function reset() public requireCracked {
        shellStrength = initialShellStrength; // Reset the shell strength.
        kernel = initialKernel; // Reset the kernel value.
        round++; // Move to the next round.
        emit Reset(round, shellStrength); // Log the reset.
    }

    // Look at the kernel if the shell is cracked and the caller contributed.
    function look() public view requireCracked onlyContributor returns (uint256) {
        return uint256(kernel); // Return the kernel value.
    }

    // Set the kernel to a specific value.
    function set_number(suint _kernel) public {
        kernel = _kernel;
    }

    // Modifier to ensure the shell is fully cracked.
    modifier requireCracked() {
        require(shellStrength == 0, "SHELL_INTACT");
        _;
    }

    // Modifier to ensure the shell is not cracked.
    modifier requireIntact() {
        require(shellStrength > 0, "SHELL_ALREADY_CRACKED");
        _;
    }

    // Modifier to ensure the caller has contributed in the current round.
    modifier onlyContributor() {
        require(hitsPerRound[round][msg.sender] > 0, "NOT_A_CONTRIBUTOR");
        _;
    }
}
```

Now, onto testing the contract!



