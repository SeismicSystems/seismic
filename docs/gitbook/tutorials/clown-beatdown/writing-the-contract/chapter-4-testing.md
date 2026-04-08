---
icon: vial-virus
---

# Ch 4: Testing

In this chapter, you'll write tests to verify that the ClownBeatdown contract behaves as expected under various scenarios. Testing ensures the functionality, fairness, and access control mechanisms of your contract work seamlessly, particularly in multi-round gameplay. _Estimated Time: ~15 minutes._

### Getting Started

Navigate to the test folder in your project and open the `ClownBeatdown.t.sol` file located at:

```bash
packages/contracts/test/ClownBeatdown.t.sol
```

This file is where you'll write all the test cases for the ClownBeatdown contract. Start with the following base code:

```solidity
// SPDX-License-Identifier: MIT License
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {ClownBeatdown} from "../src/ClownBeatdown.sol";

contract ClownBeatdownTest is Test {
    ClownBeatdown public clownBeatdown;

    function setUp() public {
        clownBeatdown = new ClownBeatdown(2);
        clownBeatdown.addSecret("Secret A");
        clownBeatdown.addSecret("Secret B");
        clownBeatdown.addSecret("Secret C");
    }
}
```

The `setUp()` function initializes the ClownBeatdown contract with a stamina of 2 and adds three secrets to the pool.

### Writing Test Cases

Start off with testing the basic functionalities: `hit`, `rob`, and `reset`.

#### Core functionalities

1. **Basic hit functionality**

Ensures the clown's stamina decreases when hit.

```solidity
function test_Hit() public {
    clownBeatdown.hit();
    assertEq(clownBeatdown.getClownStamina(), 1);
}
```

2. **Knockout and rob**

Validates that after knocking out the clown, a contributor can rob a valid secret.

```solidity
function test_KnockoutAndRob() public {
    clownBeatdown.hit();
    clownBeatdown.hit();
    // rob() should return one of the secrets
    bytes memory secret = clownBeatdown.rob();
    assertTrue(
        keccak256(secret) == keccak256(bytes("Secret A")) ||
        keccak256(secret) == keccak256(bytes("Secret B")) ||
        keccak256(secret) == keccak256(bytes("Secret C"))
    );
}
```

3. **Reset functionality**

```solidity
function test_Reset() public {
    clownBeatdown.hit();
    clownBeatdown.hit();
    clownBeatdown.reset();
    assertEq(clownBeatdown.getClownStamina(), 2); // Stamina should be reset to 2
}
```

4. **Secret can change after reset**

Validates that secrets returned across rounds are always valid (they may or may not differ depending on randomness).

```solidity
function test_SecretCanChangeAfterReset() public {
    // Knock out and rob in round 1
    clownBeatdown.hit();
    clownBeatdown.hit();
    bytes memory secret1 = clownBeatdown.rob();

    // Reset and knock out again in round 2
    clownBeatdown.reset();
    clownBeatdown.hit();
    clownBeatdown.hit();
    bytes memory secret2 = clownBeatdown.rob();

    // Both should be valid secrets (they may or may not differ depending on randomness)
    assertTrue(
        keccak256(secret1) == keccak256(bytes("Secret A")) ||
        keccak256(secret1) == keccak256(bytes("Secret B")) ||
        keccak256(secret1) == keccak256(bytes("Secret C"))
    );
    assertTrue(
        keccak256(secret2) == keccak256(bytes("Secret A")) ||
        keccak256(secret2) == keccak256(bytes("Secret B")) ||
        keccak256(secret2) == keccak256(bytes("Secret C"))
    );
}
```

Now, test for the restrictive/conditional nature of these basic functionalities.

#### Restricting Actions

1. **Preventing `hit` when clown is down**

Ensures that hitting a knocked-out clown is not allowed.

```solidity
function test_CannotHitWhenDown() public {
    clownBeatdown.hit();
    clownBeatdown.hit();
    vm.expectRevert("CLOWN_ALREADY_DOWN");
    clownBeatdown.hit();
}
```

2. **Preventing `rob` when clown is standing**

Ensures that robbing while the clown still has stamina is not allowed.

```solidity
function test_CannotRobWhenStanding() public {
    clownBeatdown.hit();
    vm.expectRevert("CLOWN_STILL_STANDING");
    clownBeatdown.rob();
}
```

3. **Preventing `reset` when clown is standing**

Validates that the clown cannot be reset unless it is fully knocked out.

```solidity
function test_CannotResetWhenStanding() public {
    vm.expectRevert("CLOWN_STILL_STANDING");
    clownBeatdown.reset();
}
```

Now, test for more complex scenarios.

#### Complex scenarios

1. **Prevent Non-Contributors From Using `rob()`**

Ensures that only contributors in the current round can call `rob()`.

```solidity
function test_RevertWhen_NonContributorTriesToRob() public {
    address nonContributor = address(0xabcd);

    // Knock out the clown
    clownBeatdown.hit();
    clownBeatdown.hit();

    // Non-contributor should be rejected
    vm.prank(nonContributor);
    vm.expectRevert("NOT_A_CONTRIBUTOR");
    clownBeatdown.rob();

    // Original contributor can still rob
    bytes memory secret = clownBeatdown.rob();
    assertTrue(secret.length > 0);
}
```

2. **Contributor Tracking Across Rounds**

Validates that contributions are tracked independently for each round. The test has one contributor knock out the clown in round 1, and a different contributor knock it out in round 2. We check that the round 2 contributor can rob while the round 1 contributor cannot.

```solidity
function test_ContributorInRound2() public {
    address contributorRound2 = address(0xabcd);

    // Round 1: knocked out by address(this)
    clownBeatdown.hit();
    clownBeatdown.hit();
    bytes memory secret1 = clownBeatdown.rob();
    assertTrue(secret1.length > 0);

    // Reset for round 2
    clownBeatdown.reset();

    // Round 2: knocked out by contributorRound2
    vm.prank(contributorRound2);
    clownBeatdown.hit();
    vm.prank(contributorRound2);
    clownBeatdown.hit();

    // contributorRound2 can rob in round 2
    vm.prank(contributorRound2);
    bytes memory secret2 = clownBeatdown.rob();
    assertTrue(secret2.length > 0);

    // address(this) cannot rob in round 2 (not a contributor this round)
    vm.expectRevert("NOT_A_CONTRIBUTOR");
    clownBeatdown.rob();
}
```

Test out the file by running the following inside the `packages/contracts` directory:

```
sforge build
sforge test
```

The contract has been tested, time to deploy it!
