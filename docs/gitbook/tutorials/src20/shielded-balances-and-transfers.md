---
description: Implement transfer() and transferFrom() with suint256
icon: arrow-right-arrow-left
---

# Shielded Balances and Transfers

This chapter walks through the full implementation of shielded transfers, allowances, and minting. You will also write tests using `sforge` to verify everything works. _Estimated time: \~20 minutes._

## Shielded balanceOf

The core change is in the mapping declaration:

```solidity
mapping(address => suint256) balanceOf;
```

At the storage level, this is where Seismic's **FlaggedStorage** comes in. Each storage slot is a tuple of `(value, is_private)`. When the compiler sees `suint256`, it emits `CSTORE` to write and `CLOAD` to read, setting the `is_private` flag to `true`. This means:

* `eth_getStorageAt` calls for these slots will fail. External observers cannot read the raw storage.
* Only `CLOAD` can access private slots. The standard `SLOAD` opcode cannot reach them.
* Anyone inspecting the state trie, transaction traces, or block data sees `0x000` in place of the actual balance.

The developer does not interact with FlaggedStorage directly. The type annotation handles everything.

## transfer()

Here is the full `transfer` implementation with shielded amounts:

```solidity
function transfer(address to, suint256 amount) public returns (bool) {
    require(balanceOf[msg.sender] >= amount, "Insufficient balance");
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
    emit Transfer(msg.sender, to, uint256(amount));
    return true;
}
```

### What happens at each stage

1. **Calldata submission** -- The user sends a Seismic transaction (type `0x4A`). The `amount` parameter is encrypted before it leaves their machine. Observers watching the mempool see `0x000` in place of the amount.
2. **Execution inside the TEE** -- The Seismic node, running inside Intel TDX, decrypts the calldata. The `require` check runs against the shielded balance. The subtraction and addition execute normally. All intermediate values involving `suint256` are shielded in the trace.
3. **Storage update** -- The new balances are written via `CSTORE`. Both the sender's and recipient's balance slots have `is_private = true`.
4. **Observer view** -- Anyone querying the contract or reading the block sees `0x000` for the amount, the sender's balance, and the recipient's balance.

Comparisons (`>=`) and arithmetic (`-=`, `+=`) work the same on `suint256` as on `uint256`. Solidity 0.8+ overflow checks also work, so if a user tries to transfer more than their balance, the transaction reverts as expected.

## transferFrom()

The full implementation with shielded allowances:

```solidity
function transferFrom(address from, address to, suint256 amount) public returns (bool) {
    require(allowance[from][msg.sender] >= amount, "Insufficient allowance");
    require(balanceOf[from] >= amount, "Insufficient balance");

    allowance[from][msg.sender] -= amount;
    balanceOf[from] -= amount;
    balanceOf[to] += amount;

    emit Transfer(from, to, uint256(amount));
    return true;
}
```

The allowance mapping stores `suint256` values:

```solidity
mapping(address => mapping(address => suint256)) allowance;
```

The pattern is identical to a standard ERC20 `transferFrom`. The only difference is the type. The allowance check, the allowance deduction, and the balance updates all use shielded arithmetic. An observer cannot see how much allowance was granted, how much was consumed, or how much remains.

## approve()

Setting shielded allowances:

```solidity
function approve(address spender, suint256 amount) public returns (bool) {
    allowance[msg.sender][spender] = amount;
    emit Approval(msg.sender, spender, uint256(amount));
    return true;
}
```

The approved amount is stored as `suint256`. No one can query how much a spender is authorized to transfer on behalf of the owner -- that information is shielded in storage. The `Approval` event above casts the amount to `uint256` for the log. If you need the approved amount to be private in the event as well, see the [Encrypted Events](encrypted-events.md) chapter.

## The mint pattern

Here is a mint function that assigns new tokens:

```solidity
function mint(address to, suint256 amount) public {
    // In production, add access control here (e.g., onlyOwner)
    totalSupply += uint256(amount);
    balanceOf[to] += amount;
    emit Transfer(address(0), to, uint256(amount));
}
```

There is a design decision here: **totalSupply is public**. It is a regular `uint256`, so the aggregate supply is visible. This is usually desirable -- users and markets want to know how many tokens exist. However, individual balances remain shielded. An observer knows the total supply increased, but cannot see which address received the tokens or how they are distributed.

If you want even the total supply to be private, you can change it to `suint256`:

```solidity
suint256 totalSupply;
```

But keep in mind that `suint256` state variables cannot be `public`, so you would need to provide a signed-read function for anyone authorized to view it.

### Constructor minting

The simplest approach is to mint the entire initial supply in the constructor:

```solidity
constructor(string memory _name, string memory _symbol, uint256 _initialSupply) {
    name = _name;
    symbol = _symbol;
    totalSupply = _initialSupply;
    balanceOf[msg.sender] = suint256(_initialSupply);
}
```

The explicit cast `suint256(_initialSupply)` is required because `_initialSupply` is a regular `uint256`. Seismic enforces explicit casting between shielded and unshielded types.

## Testing with sforge

Create a test file at `test/SRC20.t.sol`:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SRC20} from "../src/SRC20.sol";

contract SRC20Test is Test {
    SRC20 public token;
    address public alice;
    address public bob;

    function setUp() public {
        token = new SRC20("Shielded Token", "SRC", 1000000e18);
        alice = address(this);       // deployer holds initial supply
        bob = address(0xB0B);
    }
}
```

### Test: basic transfer

```solidity
function test_Transfer() public {
    // Transfer 100 tokens from alice (deployer) to bob
    bool success = token.transfer(bob, suint256(100e18));
    assertTrue(success);

    // Verify balances using the internal test helper
    // In sforge tests, the test contract can read shielded values directly
    assertEq(token.getBalanceForTest(bob), 100e18);
    assertEq(token.getBalanceForTest(alice), 999900e18);
}
```

{% hint style="info" %}
In `sforge` tests, the test contract runs inside the same execution context. You can read shielded values by adding an internal helper function to your contract (or using the test contract's access). In production, shielded values are only accessible through signed reads.
{% endhint %}

To support this test, add a test-only helper to your contract:

```solidity
// Only for testing -- remove before deployment
function getBalanceForTest(address account) external view returns (uint256) {
    return uint256(balanceOf[account]);
}
```

### Test: transfer reverts on insufficient balance

```solidity
function test_RevertWhen_InsufficientBalance() public {
    vm.prank(bob); // bob has no tokens
    vm.expectRevert("Insufficient balance");
    token.transfer(alice, suint256(1e18));
}
```

### Test: approve and transferFrom

```solidity
function test_TransferFrom() public {
    // Alice approves bob to spend 500 tokens
    token.approve(bob, suint256(500e18));

    // Bob transfers 200 tokens from alice to himself
    vm.prank(bob);
    bool success = token.transferFrom(alice, bob, suint256(200e18));
    assertTrue(success);

    // Verify balances
    assertEq(token.getBalanceForTest(bob), 200e18);
    assertEq(token.getBalanceForTest(alice), 999800e18);
}
```

### Test: transferFrom reverts on insufficient allowance

```solidity
function test_RevertWhen_InsufficientAllowance() public {
    token.approve(bob, suint256(50e18));

    vm.prank(bob);
    vm.expectRevert("Insufficient allowance");
    token.transferFrom(alice, bob, suint256(100e18));
}
```

### Running the tests

Build and run from your contracts directory:

```bash
sforge build
sforge test
```

You should see all tests passing. The shielded types behave identically to their unshielded counterparts in terms of arithmetic and comparison logic -- the privacy is handled transparently at the storage layer.
