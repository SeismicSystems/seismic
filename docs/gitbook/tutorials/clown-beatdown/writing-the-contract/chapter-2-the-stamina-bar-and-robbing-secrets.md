---
icon: hand-fist
---

# Ch 2: The Stamina Bar and Robbing Secrets

In this chapter, you'll build the **stamina bar**, the protective layer that guards the clown's secrets. You'll initialize the clown's stamina and implement a `hit` function to reduce it. Additionally, you'll add a `rob()` function with a `requireDown` modifier to ensure secrets can only be stolen once the clown is knocked out. _Estimated Time: ~10 minutes._

### Defining the stamina bar

The stamina bar determines the clown's resilience. It has an integer value (`clownStamina`), which represents how many hits the clown can withstand before going down. Let's define the stamina and initialize it in the constructor:

```solidity
    uint256 initialClownStamina; // Starting stamina restored on reset.
    uint256 clownStamina; // Remaining stamina before the clown is down.

    constructor(uint256 _clownStamina) {
        initialClownStamina = _clownStamina; // Set starting stamina.
        clownStamina = _clownStamina; // Initialize remaining stamina.
        round = 1; // Start with the first round.
    }
```

### Adding the hit function

Each time the clown is hit, its stamina decreases, bringing it one step closer to being knocked out. This is crucial for revealing secrets, **as the clown must be fully knocked out for secrets to be accessible:**

```solidity
    // Event to log hits.
    event Hit(uint256 indexed round, address indexed hitter, uint256 remaining);

    // Hit the clown to reduce stamina.
    function hit() public requireStanding {
        clownStamina--; // Decrease stamina.
        emit Hit(round, msg.sender, clownStamina); // Log the hit.
    }

    // Modifier to ensure the clown is still standing.
    modifier requireStanding() {
        require(clownStamina > 0, "CLOWN_ALREADY_DOWN");
        _;
    }
```

### What's happening here?

- **The `requireStanding` modifier**: Ensures that the function cannot be called if the clown is already knocked out (`clownStamina == 0`). This prevents unnecessary calls after the clown is down.
- **Decrementing stamina**: Each call to `hit` decreases the clown's stamina (`clownStamina`) by one.
- **Logging the action**: The `Hit` event records the round, the hitter's address (`msg.sender`), and the remaining stamina.

### Adding a stamina getter

Add a public view function so the current stamina can be checked:

```solidity
    // Get the current clown stamina.
    function getClownStamina() public view returns (uint256) {
        return clownStamina;
    }
```

### Robbing the clown

Now that we have implemented the stamina system and the ability to reduce it using the hit function, we can introduce the robbery mechanic: the clown's secret should only be revealed once it is fully knocked out.

- The secret **should remain hidden** while the **clown is standing**.
- The secret **can only be stolen** once the clown's stamina reaches **zero, i.e. when it is knocked out**.

To enforce this, we will create a function called `rob()`, which will return a randomly selected secret, but only if the clown has been fully knocked out.

Here's how we define `rob()` with a `requireDown` modifier:

```solidity
    // Reveal secret once the clown is down and the caller contributed.
    function rob() public view requireDown returns (bytes memory) {
        sbytes memory secret = secrets[uint256(secretIndex)];
        return bytes(secret); // Return the randomly selected secret.
    }

    // Modifier to ensure the clown is down.
    modifier requireDown() {
        require(clownStamina == 0, "CLOWN_STILL_STANDING");
        _;
    }
```

### What's happening here?

- **Restricting Access with a Condition**: The `requireDown` modifier ensures that `rob()` can only be called if `clownStamina == 0`, meaning the clown has been fully knocked out.
- **Revealing the Secret**: Once the condition is met, `rob()` reads the secret at the shielded `secretIndex` position. It converts the `sbytes` value to plain `bytes` for the caller.
- **Preventing Premature Access**: If `rob()` is called before the clown is knocked out, the function will revert with the error `"CLOWN_STILL_STANDING"`.

### Updated contract with hit and rob

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

    // Event to log hits.
    event Hit(uint256 indexed round, address indexed hitter, uint256 remaining);

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
        emit Hit(round, msg.sender, clownStamina); // Log the hit.
    }

    // Reveal secret once the clown is down.
    function rob() public view requireDown returns (bytes memory) {
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
}
```
