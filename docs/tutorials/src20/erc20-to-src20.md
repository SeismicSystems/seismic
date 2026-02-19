---
description: "See exactly what changes between a standard ERC20 and a private SRC20"
icon: right-left
---

# ERC20 to SRC20: What Changes

This chapter puts a standard ERC20 and its SRC20 counterpart side by side so you can see exactly how little changes. The core insight: privacy on Seismic is a type-level change, not an architectural one. _Estimated time: ~10 minutes._

## The standard ERC20

Here is a minimal but complete ERC20 contract. It covers the full interface -- name, symbol, decimals, totalSupply, balances, allowances, transfer, approve, and transferFrom:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract ERC20 {
    string public name;
    string public symbol;
    uint8 public decimals = 18;
    uint256 public totalSupply;

    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);

    constructor(string memory _name, string memory _symbol, uint256 _initialSupply) {
        name = _name;
        symbol = _symbol;
        totalSupply = _initialSupply;
        balanceOf[msg.sender] = _initialSupply;
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        require(balanceOf[msg.sender] >= amount, "Insufficient balance");
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        emit Transfer(msg.sender, to, amount);
        return true;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        require(allowance[from][msg.sender] >= amount, "Insufficient allowance");
        require(balanceOf[from] >= amount, "Insufficient balance");
        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }
}
```

This is standard Solidity. Every balance, transfer amount, and allowance is a `uint256`, visible to anyone who queries the contract or reads the chain.

## The SRC20 version

Here is the same contract converted to an SRC20. Changed lines are marked with comments:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract SRC20 {
    string public name;
    string public symbol;
    uint8 public decimals = 18;
    uint256 public totalSupply;                                          // stays public

    mapping(address => suint256) balanceOf;                              // CHANGED: uint256 -> suint256, removed public
    mapping(address => mapping(address => suint256)) allowance;          // CHANGED: uint256 -> suint256, removed public

    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);

    constructor(string memory _name, string memory _symbol, uint256 _initialSupply) {
        name = _name;
        symbol = _symbol;
        totalSupply = _initialSupply;
        balanceOf[msg.sender] = suint256(_initialSupply);                // CHANGED: cast to suint256
    }

    function transfer(address to, suint256 amount) public returns (bool) {   // CHANGED: uint256 -> suint256
        require(balanceOf[msg.sender] >= amount, "Insufficient balance");
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        emit Transfer(msg.sender, to, uint256(amount));                  // CHANGED: cast back for event
        return true;
    }

    function approve(address spender, suint256 amount) public returns (bool) {  // CHANGED: uint256 -> suint256
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, uint256(amount));             // CHANGED: cast back for event
        return true;
    }

    function transferFrom(address from, address to, suint256 amount) public returns (bool) {  // CHANGED: uint256 -> suint256
        require(allowance[from][msg.sender] >= amount, "Insufficient allowance");
        require(balanceOf[from] >= amount, "Insufficient balance");
        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        emit Transfer(from, to, uint256(amount));                        // CHANGED: cast back for event
        return true;
    }
}
```

That is the entire diff. The contract logic is structurally identical.

## Line-by-line diff

### Balances: `uint256` to `suint256`

```diff
- mapping(address => uint256) public balanceOf;
+ mapping(address => suint256) balanceOf;
```

Two changes here. First, the value type changes from `uint256` to `suint256`. This tells the Seismic compiler to emit `CSTORE`/`CLOAD` instead of `SSTORE`/`SLOAD`, which marks these storage slots as private. Second, the `public` visibility modifier is removed. Shielded types cannot be returned from public or external functions, so the automatic getter that `public` generates would not compile. We will add an explicit balance-checking function using signed reads in a later chapter.

### Allowances: same pattern

```diff
- mapping(address => mapping(address => uint256)) public allowance;
+ mapping(address => mapping(address => suint256)) allowance;
```

The same change applies to the allowance mapping. The nested mapping's value type becomes `suint256`, and the `public` modifier is removed.

### Function parameters: shielded amounts

```diff
- function transfer(address to, uint256 amount) public returns (bool) {
+ function transfer(address to, suint256 amount) public returns (bool) {
```

The `amount` parameter changes to `suint256`. When a user calls this function through a Seismic transaction (type `0x4A`), the amount is encrypted in the calldata before it leaves their machine. During execution inside the TEE, the amount is decrypted and used normally. Observers watching the mempool or block data see `0x000` in place of the amount.

The same change applies to `approve` and `transferFrom`.

### Constructor: casting the initial supply

```diff
- balanceOf[msg.sender] = _initialSupply;
+ balanceOf[msg.sender] = suint256(_initialSupply);
```

Since `_initialSupply` is a regular `uint256` (it is the total supply, which is public) and `balanceOf` now stores `suint256` values, an explicit cast is required. Seismic does not allow implicit casting between shielded and unshielded types.

### Events: casting back to `uint256`

```diff
- emit Transfer(msg.sender, to, amount);
+ emit Transfer(msg.sender, to, uint256(amount));
```

Events are stored in transaction logs, which are public. Shielded types cannot appear in event parameters. The simplest approach is to cast the amount back to `uint256` before emitting. Note that this **does** reveal the amount in the event log. If you need the event data to also be private, the [Encrypted Events](encrypted-events.md) chapter shows how to use AES-GCM precompiles to encrypt it.

## What stays the same

A lot stays the same, which is the point:

- **Transfer logic** -- The subtraction, addition, and require checks are identical. Arithmetic operations on `suint256` work the same as on `uint256`.
- **Overflow protection** -- Solidity 0.8+ overflow checks work with shielded types.
- **Function signatures** -- The function names and return types are unchanged. The contract is still recognizable as an ERC20.
- **Address parameters** -- The `to`, `from`, and `spender` parameters remain regular `address` types. You could shield these with `saddress` if you wanted to hide the participants, but for a standard token that is usually not needed.
- **totalSupply** -- This stays as a regular `uint256`. The total supply is public information. Individual balances are private, but the aggregate is visible.

## What you lose (and how to get it back)

There are two capabilities that the basic SRC20 loses compared to a standard ERC20:

### 1. Public balance queries

Since `balanceOf` cannot be `public`, there is no automatic getter. Users cannot call `balanceOf(address)` the way they would with a standard ERC20. The solution is **signed reads** -- a Seismic-specific mechanism where a user sends a signed `eth_call` (type `0x4A`) to prove their identity, and the contract returns their balance only to them. This is covered in [Signed Reads for Balance Checking](signed-reads-for-balance-checking.md).

### 2. Private event data

With the simple cast approach above, the amount appears in plaintext in the event log. If you need transfer amounts to be hidden in events as well, you can encrypt the data using Seismic's AES-GCM precompiles before emitting. This is covered in [Encrypted Events](encrypted-events.md).

Both of these are straightforward additions. The next chapters walk through each one.
