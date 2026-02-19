---
icon: vial-virus
---

# Chapter 4: Testing your Walnut contract

In this chapter, you’ll write tests to verify that the Walnut contract behaves as expected under various scenarios. Testing ensures the functionality, fairness, and access control mechanisms of your contract work seamlessly, particularly in multi-round gameplay. _Estimated Time: \~15 minutes._

### Getting Started

Navigate to the test folder in your Walnut App and open the `Walnut.t.sol` file located at:

```bash
packages/contracts/test/Walnut.t.sol
```

This file is where you’ll write all the test cases for the Walnut contract. Start with the following base code:

```solidity
// SPDX-License-Identifier: MIT License
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Walnut} from "../src/Walnut.sol";

contract WalnutTest is Test {
    Walnut public walnut;

    function setUp() public {
        // Initialize a Walnut with shell strength = 2 and kernel = 0
        walnut = new Walnut(2, suint256(0));
    }
}
```

The `setUp()` function initializes the Walnut contract for use in all test cases.

### Writing Test Cases

Start off with testing the basic functionalities, `hit` , `shake` , `look` and `reset`

#### Core functionalities

1. **Basic hit functionality**

Ensures the Walnut’s shell can be cracked by `shellStrength` number of hits.

```solidity
function test_Hit() public {
    walnut.hit(); // Decrease shell strength by 1
    walnut.hit(); // Fully crack the shell
    assertEq(walnut.look(), 0); // Kernel should still be 0 since no shakes
}
```

2. **Basic shake functionality**

Validates that shaking the Walnut increments the kernel value.

```solidity
function test_Shake() public {
    walnut.shake(suint256(10)); // Shake the Walnut, increasing the kernel
    walnut.hit(); // Decrease shell strength by 1
    walnut.hit(); // Fully crack the shell
    assertEq(walnut.look(), 10); // Kernel should be 10 after 10 shakes
}
```

3. **Reset functionality**

```solidity
function test_Reset() public {
    walnut.hit(); // Decrease shell strength by 1
    walnut.shake(suint256(2)); // Shake the Walnut
    walnut.hit(); // Fully crack the shell
    walnut.reset(); // Reset the Walnut

    assertEq(walnut.getShellStrength(), 2); // Shell strength should reset to initial value
    walnut.hit(); // Start hitting again
    walnut.shake(suint256(5)); // Shake the Walnut again
    walnut.hit(); // Fully crack the shell again
    assertEq(walnut.look(), 5); // Kernel should reflect the shakes in the new round
}
```

Now, test for the restrictive/conditional nature of these basic functionalities.

#### Restricting Actions

1. **Preventing `hit` when shell is cracked**

Ensures that hitting a cracked shell is not allowed.

```solidity
function test_CannotHitWhenCracked() public {
    walnut.hit(); // Decrease shell strength by 1
    walnut.hit(); // Fully crack the shell
    vm.expectRevert("SHELL_ALREADY_CRACKED"); // Expect revert when hitting an already cracked shell
    walnut.hit();
}
```

2. **Preventing `shake` when shell is cracked**

Ensures that shaking the Walnut after the shell is cracked is not allowed.

```solidity
function test_CannotShakeWhenCracked() public {
    walnut.hit(); // Decrease shell strength by 1
    walnut.shake(suint256(1)); // Shake the Walnut
    walnut.hit(); // Fully crack the shell
    vm.expectRevert("SHELL_ALREADY_CRACKED"); // Expect revert when shaking an already cracked shell
    walnut.shake(suint256(1));
}
```

3. **Preventing `look` when shell is intact**

Ensures that the kernel cannot be revealed unless the shell is fully cracked.

```solidity
function test_CannotLookWhenIntact() public {
    walnut.hit(); // Partially crack the shell
    walnut.shake(suint256(1)); // Shake the Walnut
    vm.expectRevert("SHELL_INTACT"); // Expect revert when trying to look at the kernel with the shell intact
    walnut.look();
}
```

4. **Preventing `reset` when shell is intact**

Validates that the Walnut cannot be reset unless the shell is fully cracked.

```solidity
function test_CannotResetWhenIntact() public {
    walnut.hit(); // Partially crack the shell
    walnut.shake(suint256(1)); // Shake the Walnut
    vm.expectRevert("SHELL_INTACT"); // Expect revert when trying to reset without cracking the shell
    walnut.reset();
}
```

Now, test for more complex scenarios.

#### Complex scenarios

1. **Sequence of Multiple Actions**

Ensures that the Walnut behaves correctly under a sequence of hits and shakes.

```solidity
function test_ManyActions() public {
    uint256 shakes = 0;
    for (uint256 i = 0; i < 50; i++) {
        if (walnut.getShellStrength() > 0) {
            if (i % 25 == 0) {
                walnut.hit(); // Hit the shell every 25 iterations
            } else {
                uint256 numShakes = (i % 3) + 1; // Random shakes between 1 and 3
                walnut.shake(suint256(numShakes));
                shakes += numShakes;
            }
        }
    }
    assertEq(walnut.look(), shakes); // Kernel should match the total number of shakes
}
```

2. **Prevent Non-Contributors From Using `look()`**

Ensures that only contributors in the current round can call `look()` .

```solidity
function test_RevertWhen_NonContributorTriesToLook() public {
    address nonContributor = address(0xabcd);

    walnut.hit(); // Decrease shell strength by 1
    walnut.shake(suint256(3)); // Shake the Walnut
    walnut.hit(); // Fully crack the shell

    vm.prank(nonContributor); // Impersonate a non-contributor
    vm.expectRevert("NOT_A_CONTRIBUTOR"); // Expect revert when non-contributor calls `look()`
    walnut.look();
}
```

3. **Contributor Tracking Across Rounds**

Validates that contributions are tracked independently for each round. The test has one contributor hit both times and crack the shell in the first round, and a different contributor hit and crack the shell in the second round. We check for the fact the second round contributor cannot see the kernel after the first round and the first round contributor cannot see the kernel after the second.

```solidity
function test_ContributorInRound2() public {
    address contributorRound2 = address(0xabcd); // Contributor for round 2

    // Round 1: Cracked by address(this)
    walnut.hit(); // Hit 1
    walnut.hit(); // Hit 2
    assertEq(walnut.look(), 0); // Confirm kernel value

    walnut.reset(); // Start Round 2

    // Round 2: ContributorRound2 cracks the Walnut
    vm.prank(contributorRound2);
    walnut.hit();

    vm.prank(contributorRound2);
    walnut.shake(suint256(5)); // Shake kernel 5 times

    vm.prank(contributorRound2);
    walnut.hit();

    vm.prank(contributorRound2);
    assertEq(walnut.look(), 5); // Kernel value is 5 for contributorRound2

    vm.expectRevert("NOT_A_CONTRIBUTOR"); // address(this) cannot look in round 2
    walnut.look();
}
```

You can find the entire test file [here](https://github.com/SeismicSystems/seismic-starter/blob/6251e663814ad9018441b15edc6a3a83fd9d38ce/packages/contracts/test/Walnut.t.sol#L1).

Test out the file by running the following inside the `packages/contracts` directory:

```
sforge build
sforge test
```

The contract has been tested, time to deploy it!
