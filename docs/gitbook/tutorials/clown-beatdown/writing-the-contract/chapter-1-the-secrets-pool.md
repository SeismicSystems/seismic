---
icon: face-clown
---

# Ch 1: The Secrets Pool

In this chapter, you'll learn to create and initialize the secrets pool — a collection of hidden strings stored inside the clown's pockets — and implement a function to add new secrets. _Estimated time: ~10 minutes._

### Defining the secrets pool

The **secrets pool** is the collection of hidden strings that the clown carries. Using Seismic's **`sbytes`** type, each secret is shielded on-chain — encrypted and invisible to observers. A shielded **`suint256`** index determines which secret gets revealed when the clown is robbed. Open up `packages/contracts/ClownBeatdown.sol` and define the state variables:

```solidity
// SPDX-License-Identifier: MIT License
pragma solidity ^0.8.13;

contract ClownBeatdown {
    mapping(uint256 => sbytes) secrets; // Pool of possible secrets (shielded).
    uint256 secretsCount; // Number of secrets for modular arithmetic.
    suint256 secretIndex; // Shielded index into the secrets mapping.
    uint256 round; // The current round number (used by _randomIndex).

    constructor(uint256 _clownStamina) {
        round = 1; // Start with the first round.
    }
}
```

### Add the addSecret function

Next, let's implement a function to add secrets to the pool. The `addSecret` function takes a plain `string` and converts it to `sbytes` for shielded storage:

```solidity
function addSecret(string memory _secret) public {
    secrets[secretsCount] = sbytes(_secret);
    secretsCount++;
    secretIndex = suint256(_randomIndex()); // Re-pick a random secret.
}
```

### Add the random index helper

The `_randomIndex` function generates a pseudo-random index into the secrets array using on-chain randomness sources:

```solidity
// Generate a pseudo-random index into the secrets array.
function _randomIndex() private view returns (uint256) {
    return uint256(keccak256(abi.encodePacked(block.prevrandao, block.timestamp, round))) % secretsCount;
}
```

### What's happening here?

The `addSecret` function converts a plain `string` into `sbytes` (shielded bytes) and stores it in the `secrets` mapping. Because `sbytes` is a shielded type, the secret's contents are encrypted on-chain and invisible to observers.

The function also updates shielded state (`secretIndex` is a `suint256`), which makes a call to this function a **shielded write.**

Note that we're using two different shielded types here:

- **`sbytes`** — for storing encrypted strings (the secrets themselves)
- **`suint256`** — for storing an encrypted integer (the index that determines which secret gets revealed)
