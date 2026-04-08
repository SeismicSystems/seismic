---
icon: people-line
---

# Ch 3: Rounds and Contributor Access

In this chapter, we'll implement a reset mechanism that allows the clown to get back up for multiple rounds, ensuring each game session starts fresh. We'll also track contributors per round so that only players who participated in knocking out the clown can call `rob()`. By the end, we'll have a fully functional round-based game where secrets remain shielded until conditions are met! _Estimated time: ~15 minutes._

### The need for a Reset mechanism

Right now, once the clown is knocked out, there's no way to reset it. If a game session were to continue, we'd have no way to start fresh — the stamina would remain at 0, and the secret would be permanently accessible.

To solve this, we need to introduce:

- A `reset` function that restores the clown to its original state.
- Round tracking, so each reset creates a new round.

### The need for a contributor check

While the reset mechanism and round tracking allow us to restart the game for continuous play, they still don't address **who** should be allowed to call the `rob()` function.

Right now, any player can call `rob()` once the clown is knocked out, even if they didn't participate in hitting it during the current round. This creates the following issues:

- **Fairness**: Players who didn't contribute should not be able to reap the benefits of robbing the clown.
- **Incentivizing Contribution**: The game needs to encourage active participation by ensuring that only those who helped knock out the clown in a specific round are rewarded with access to the secret.

The solution to this is implementing a conditional check on `rob()` which allows only those players who **contributed at least one hit in the current round** to steal the secret.

### Implementing the Reset Mechanism

The reset mechanism allows the clown to get back up for multiple rounds, with each round starting fresh. It restores the clown's stamina and picks a new random secret, then increments the round counter.

Here's how we can implement the reset function:

```solidity
    // Event to log resets.
    event Reset(uint256 indexed newRound, uint256 remainingClownStamina);

    // Reset the beatdown for a new round.
    function reset() public requireDown {
        clownStamina = initialClownStamina; // Reset stamina.
        secretIndex = suint256(_randomIndex()); // Pick a new random secret.
        round++; // Move to the next round.
        emit Reset(round, clownStamina); // Log the reset.
    }
```

**What's Happening Here?**

- **Condition for Reset (`requireDown`):** The reset function can only be called once the clown is knocked out, enforced by the `requireDown` modifier.
- **Restoring Initial State**: The stamina is reset to `initialClownStamina`, and a new random secret is selected via `_randomIndex()`.
- **Round Tracking**: The `round` counter increments each time the clown is reset, allowing us to distinguish between rounds.

### Modifying hit() to track contributions

To enforce fair access to the secrets, we'll track the number of hits each player contributes in a given round. This is achieved using the `hitsPerRound` mapping:

```solidity
    // Tracks the number of hits per player per round.
    mapping(uint256 => mapping(address => uint256)) hitsPerRound;
```

Every time a player calls the `hit()` function, we update their contribution in the current round:

```solidity
    // Hit the clown to reduce stamina.
    function hit() public requireStanding {
        clownStamina--; // Decrease stamina.
        hitsPerRound[round][msg.sender]++; // Record the player's hit for the current round.
        emit Hit(round, msg.sender, clownStamina); // Log the hit.
    }
```

**What's Happening Here?**

- **Tracking Contributions**: The `hitsPerRound` mapping records each player's hits in the current round. This ensures we can verify who participated when the clown was knocked out.
- **Replayable Rounds**: Because contributions are tracked by round, the game can fairly reset and start fresh without losing player data from previous rounds.

### Restricting rob() with a contributor check

To ensure only contributors can rob the clown, we'll use a modifier called `onlyContributor`:

```solidity
    // Modifier to ensure the caller has contributed in the current round.
    modifier onlyContributor() {
        require(hitsPerRound[round][msg.sender] > 0, "NOT_A_CONTRIBUTOR");
        _;
    }
```

We'll then apply this modifier to the `rob()` function:

```solidity
    // Reveal secret once the clown is down and the caller contributed.
    function rob() public view requireDown onlyContributor returns (bytes memory) {
        sbytes memory secret = secrets[uint256(secretIndex)];
        return bytes(secret); // Return the randomly selected secret.
    }
```

**Congratulations!** You made it through writing the entire shielded smart contract for a multiplayer, multi-round, Clown Beatdown game!

**Final ClownBeatdown contract**

```solidity
// SPDX-License-Identifier: MIT License
pragma solidity ^0.8.13;

contract ClownBeatdown {
    uint256 initialClownStamina; // Starting stamina restored on reset.
    uint256 clownStamina; // Remaining stamina before the clown is down.
    uint256 round; // The current round number.

    mapping(uint256 => sbytes) secrets; // Pool of possible secrets (shielded).
    uint256 secretsCount; // Number of secrets for modular arithmetic.
    suint256 secretIndex; // Shielded index into the secrets mapping.

    // Tracks the number of hits per player per round.
    mapping(uint256 => mapping(address => uint256)) hitsPerRound;

    // Events to log hits and resets.

    // Event to log hits.
    event Hit(uint256 indexed round, address indexed hitter, uint256 remaining); // Logged when a hit lands.
    // Event to log resets.
    event Reset(uint256 indexed newRound, uint256 remainingClownStamina);

    constructor(uint256 _clownStamina) {
        initialClownStamina = _clownStamina; // Set starting stamina.
        clownStamina = _clownStamina; // Initialize remaining stamina.
        round = 1; // Start with the first round.
    }

    // Get the current clown stamina.
    function getClownStamina() public view returns (uint256) {
        return clownStamina;
    }

    function addSecret(string memory _secret) public {
        secrets[secretsCount] = sbytes(_secret);
        secretsCount++;
        secretIndex = suint256(_randomIndex()); // Re-pick a random secret.
    }

    // Hit the clown to reduce stamina.
    function hit() public requireStanding {
        clownStamina--; // Decrease stamina.
        hitsPerRound[round][msg.sender]++; // Record the player's hit for the current round.
        emit Hit(round, msg.sender, clownStamina); // Log the hit.
    }


    // Reset the beatdown for a new round.
    function reset() public requireDown {
        clownStamina = initialClownStamina; // Reset stamina.
        secretIndex = suint256(_randomIndex()); // Pick a new random secret.
        round++; // Move to the next round.
        emit Reset(round, clownStamina); // Log the reset.
    }

    // Reveal secret once the clown is down and the caller contributed.
 function rob() public view requireDown onlyContributor returns (bytes memory) {
        sbytes memory secret = secrets[uint256(secretIndex)];
        return bytes(secret); // Return the randomly selected secret.
    }

    // Generate a pseudo-random index into the secrets array.
    function _randomIndex() private view returns (uint256) {
        return uint256(keccak256(abi.encodePacked(block.prevrandao, block.timestamp, round))) % secretsCount;
    }

    // Modifier to ensure the clown is down.
    modifier requireDown() {
        require(clownStamina == 0, "CLOWN_STILL_STANDING");
        _;
    }

    // Modifier to ensure the clown is still standing.
    modifier requireStanding() {
        require(clownStamina > 0, "CLOWN_ALREADY_DOWN");
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
