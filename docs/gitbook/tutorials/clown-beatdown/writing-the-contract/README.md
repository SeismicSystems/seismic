---
icon: file-signature
---

# Writing the Contract

This section dives into the heart of Clown Beatdown — the **shielded smart contract** that powers its functionality. You'll start by building the secrets pool using shielded storage, then implement the stamina system and the robbery mechanic, before adding rounds, resets, and contributor-based access control. By the end of this section, you'll have a fully functional, round-based ClownBeatdown contract that is secure, fair, and replayable.

### What You'll Learn

In this section, you'll:

- Define a pool of **shielded secrets** using `sbytes` and a shielded index using `suint256`.
- Build the **stamina bar**, the protective layer that guards the secrets, and implement a `hit()` function to reduce it.
- Add a `rob()` function to reveal a randomly selected secret to authorized contributors.
- Implement a reset mechanism to restart the game for multiple rounds.
- Track player contributions in each round, ensuring that only contributors can rob the clown.

### Overview of Chapters

- [**Chapter 1: The Secrets Pool**](chapter-1-the-secrets-pool.md)

You'll define the secrets pool using shielded storage (`sbytes`) and a shielded index (`suint256`), and implement an `addSecret()` function to populate it. This chapter introduces **shielded writes.**

- [**Chapter 2: The Stamina Bar and Robbing Secrets**](chapter-2-the-stamina-bar-and-robbing-secrets.md)

Learn how to build the stamina system, which protects the secrets from being accessed prematurely. You'll implement the `hit()` function to reduce stamina and the `rob()` function to reveal a secret once conditions are met.

- [**Chapter 3: Reset Mechanism, Rounds, and Contributor Access**](chapter-3-reset-mechanism-rounds-and-contributor-access.md)

This chapter introduces a reset mechanism to enable multiple rounds of gameplay. You'll track contributions per round and ensure that only players who helped knock out the clown in a specific round can rob it. This chapter introduces **signed reads.**
