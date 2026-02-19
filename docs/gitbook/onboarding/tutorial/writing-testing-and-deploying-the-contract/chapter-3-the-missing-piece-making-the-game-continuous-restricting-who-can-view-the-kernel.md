---
hidden: true
icon: puzzle
---

# Chapter 3: The Missing Piece - Making the Game Continuous Restricting Who Can View The Kernel

In this chapter, you will think more about _who_ can view the kernel, and how we can make the game continuous, i.e. not end after the walnut is cracked. _Estimated time: \~5 minutes_

Right now, anyone can call `look()` once the shell is cracked. While we’ve successfully gated access based on the shell’s state, this doesn’t fully protect the kernel’s visibility.

### What's missing

• The kernel is only conditionally shielded—it remains hidden while the shell is intact, but **once cracked, anyone can read it.**

• In a competitive or interactive game, we only want players who performed a certain action, such as **contributing to breaking the shell,** to be able to see the kernel’s value.

• Additionally, once a Walnut is cracked, there’s no way to reset it—the game would end after one round.

### Introducing Rounds and resetting the Walnut

To enable continuous gameplay, we need a way to restore the Walnut to its original state after it has been cracked. This means:

✅ Implementing a reset mechanism that starts a new round, **resetting the shell and kernel to their initial values.**

✅ Keeping track of who contributed in cracking the walnut (i.e., called `hit()` at least once) in a **particular round**, so that only those players are able to view the kernel value after the walnut has been cracked in that round.

Each round should function independently:

1\. **Round starts** → The Walnut has an intact shell and a hidden kernel.

2\. **Players hit the shell** → Once it reaches zero, the Walnut is cracked.

3\. **Only contributors can reveal the kernel** → Players who helped break the shell can call look().

4\. **Reset the Walnut** → The shell and kernel return to their original values, and a new round begins.

This ensures **fairness** (only contributors can see the kernel) and **replayability** (the game doesn’t end after one round).
