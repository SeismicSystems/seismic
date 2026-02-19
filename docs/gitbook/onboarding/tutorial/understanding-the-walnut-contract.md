---
icon: acorn
---

# Understanding the Walnut contract

Imagine you’re holding a walnut, an unassuming object. Inside it lies a number, a secret **only revealed when you crack the shell**. There are primarily two actions you can take on this walnut: either **shaking** it or **hitting** it. **Shaking** the walnut some `n`  number of times increments the number inside by `n`, while **hitting** the walnut brings it one step closer to the shell cracking. **You can only see the number inside if you have contributed to the cracking the shell, i.e., you have hit the walnut at least once.** This collaborative challenge is the heart of the Walnut App, and it’s all powered by the Walnut smart contract. Let’s dive into the inner workings of the contract and uncover how each part fuels this game.

The contract can be found in the `packages/contracts/Walnut.sol` file of the starter repo.

## State variables

### startShell and shell

Think of the shell as the Walnut’s durability. `startShell` is the Walnut’s starting strength, and `shell` tracks how much of it remains as players hit it.&#x20;

### startNumber and number

These are the secret numbers at the heart of the Walnut. `startNumber` initializes the hidden `number`, while `number` evolves as players shake the Walnut. Being `suint256` (shielded integers), these numbers remain encrypted on-chain—visible only to authorized participants.

### round

A counter that increments with each new round/reset, ensuring every round has a fresh Walnut to crack.

### hitsPerRound

A mapping that records every player’s contribution to the current round, ensuring only participants can peek at the Walnut’s secret.



## Functions

### hit ( )&#x20;

This function allows a player to hit the Walnut, reducing its durability and bringing it one step closer to cracking:

**What happens:**

* Checks if the shell is intact (`shell>0` )
* If it is, decrements `shell` by 1
* Increases the player who called `hit()`  's contribution in the current round (`hitsPerRound[round][playerAddress])` by 1
* Emits the `Hit`  event to update all participants.

### shake (suint256 \_numShakes)

This function allows a player to shake the walnut `_numShakes` number of times. Since this is a **write** function that takes in an `stype`  as one of its parameters, calling this function would constitute a **Seismic write.**

**What happens:**

* Adds `_numShakes` to `number`&#x20;
* Emits the `Shake` event.

### look ( )

This function allows contributors to the current round to view the `number`  inside the walnut. Since this is a **view** function that **reveals an `stype`,** calling this function would constitute a **Seismic read.**

**What happens:**

* Requires the shell to be cracked (`requireCracked` modifier)
* Ensures the function caller contributed to the cracking the walnut for this round (`onlyContributor` modifier)
* Returns ("reveals") the `number` inside the walnut.



## Modifiers

Modifiers enforce the rules of the game:

### requireCracked

Ensures that `look()` can only be called if the Walnut’s shell is completely cracked.

### requireIntact

Ensures that `shake()` and `hit()` can only be called if the Walnut’s shell is intact.

### onlyContributor

Restricts access to `look()` , and hence the `number` being revealed, only to players who contributed **at least one hit in the current round.**
