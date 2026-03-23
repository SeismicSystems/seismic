---
icon: face-clown
---

# Understanding the ClownBeatdown contract

Imagine a clown standing in front of you, taunting the crowd. Hidden in the clown's pockets are **secrets** — strings that **only become visible once the clown is knocked out**. There are primarily two actions you can take: **hitting** the clown or **adding secrets** to the clown's pockets. **Hitting** the clown reduces its stamina by one, bringing it closer to being knocked out. **Adding a secret** stores an encrypted string that no one can read until it's revealed. **You can only rob the clown of a secret if you contributed to knocking it out, i.e., you hit the clown at least once in the current round.** This collaborative challenge is the heart of the Clown Beatdown game, and it's all powered by the ClownBeatdown smart contract. Let's dive into the inner workings of the contract and uncover how each part fuels this game.

The contract can be found in the `contracts/src/ClownBeatdown.sol` file of the starter repo.

## State variables

### initialClownStamina and clownStamina

Think of stamina as the clown's durability. `initialClownStamina` is the clown's starting strength, and `clownStamina` tracks how much remains as players hit it.

### secrets, secretsCount, and secretIndex

These are the hidden values at the heart of the game. `secrets` is a mapping of `sbytes` (shielded bytes) — encrypted strings stored on-chain that remain hidden until revealed. `secretsCount` tracks how many secrets have been added. `secretIndex` is a `suint256` (shielded integer) that determines which secret will be revealed when the clown is robbed. Because `secretIndex` is shielded, no one can predict which secret will be returned.

### round

A counter that increments with each new round/reset, ensuring every round has a fresh clown to beat down.

### hitsPerRound

A mapping that records every player's contribution to the current round, ensuring only participants can rob the clown's secret.

## Functions

### addSecret (string \_secret)

This function allows anyone to add a secret to the clown's pool. Since `addSecret` converts a plain `string` into `sbytes` (shielded bytes) and stores it in the `secrets` mapping, it performs a **shielded write** — the secret's contents are encrypted on-chain and invisible to observers.

**What happens:**

- Converts the input string to `sbytes` and stores it in the `secrets` mapping
- Increments `secretsCount`
- Re-picks a random `secretIndex` using `_randomIndex()`

### hit ( )

This function allows a player to hit the clown, reducing its stamina and bringing it one step closer to being knocked out:

**What happens:**

- Checks if the clown is still standing (`clownStamina > 0`)
- If it is, decrements `clownStamina` by 1
- Increases the player who called `hit()`'s contribution in the current round (`hitsPerRound[round][playerAddress]`) by 1
- Emits the `Hit` event to update all participants.

### rob ( )

This function allows contributors to the current round to steal a randomly selected secret from the clown. Since this is a **view** function that **reveals an `sbytes` value**, calling this function constitutes a **signed read.**

**What happens:**

- Requires the clown to be knocked out (`requireDown` modifier)
- Ensures the function caller contributed to knocking out the clown for this round (`onlyContributor` modifier)
- Returns the secret at the shielded `secretIndex` position, decrypted from `sbytes` to `bytes`.

### reset ( )

This function resets the clown for a new round of gameplay:

**What happens:**

- Requires the clown to be knocked out (`requireDown` modifier)
- Restores `clownStamina` to `initialClownStamina`
- Picks a new random `secretIndex`
- Increments the `round` counter
- Emits the `Reset` event.

## Modifiers

Modifiers enforce the rules of the game:

### requireDown

Ensures that `rob()` and `reset()` can only be called if the clown's stamina has reached zero.

### requireStanding

Ensures that `hit()` can only be called while the clown still has stamina remaining.

### onlyContributor

Restricts access to `rob()`, and hence the secret being revealed, only to players who contributed **at least one hit in the current round.**
