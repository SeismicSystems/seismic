---
icon: file-signature
---

# Writing, testing and deploying the contract

This section dives into the heart of the Walnut App—the **shielded smart contract** that powers its functionality. You’ll start by building the foundational pieces of the Walnut, including the kernel and the protective shell, before implementing more advanced features like rounds, reset mechanisms, and contributor-based access control. By the end of this section, you’ll have a fully functional, round-based Walnut contract that is secure, fair, and replayable.

### What You'll Learn

In this section, you’ll:

• Define and initialize the kernel, the hidden value inside the Walnut.

• Build the **shell**, the protective layer that hides the kernel, and implement a `hit()` function to help crack it, and a `shake` function to increment the kernel value by an encrypted amount.

• Add a `look()` function to conditionally reveal the kernel.

• Implement a reset mechanism to restart the Walnut for multiple rounds.

• Track player contributions in each round, ensuring that only contributors can access the kernel.

### Overview of Chapters

* [**Chapter 1: Making the Kernel**](chapter-1-making-the-kernel.md)

You’ll define the kernel using a shielded state variable (suint256) and implement a shake() function to increment its value. This chapter introduces **shielded writes.**

* [**Chapter 2: Making the Shell and Revealing the Kernel**](chapter-2-making-the-shell.md)

Learn how to build the shell, which protects the kernel from being accessed prematurely. You’ll implement the `hit()`function to crack the shell and the `look()` function to reveal the kernel once conditions are met.

* [**Chapter 3: Reset Mechanism, Rounds, and a more conditional Kernel Reveal**](chapter-3-rounds-and-access-control.md)

This chapter introduces a reset mechanism to enable multiple rounds of gameplay. You’ll track contributions per round and ensure that only players who helped crack the Walnut in a specific round can reveal the kernel. This chapter introduces **shielded reads.**
