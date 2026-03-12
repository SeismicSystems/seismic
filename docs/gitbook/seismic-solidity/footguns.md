---
icon: triangle-exclamation
---

# Footguns

Shielded types protect values at rest and in transit, but careless usage patterns can leak information through side channels. This page covers the most common information leak vectors when working with shielded types.

### Conditional Execution

**The problem:** Using an `sbool` in a conditional branch leaks information through the execution trace and gas consumption. Observers can tell which branch was taken by examining the gas used or the operations performed.

```solidity
// BAD: Leaks the value of `isVIP` via gas difference
sbool isVIP = /* ... */;
if (isVIP) {
    discount = 50s;
} else {
    // extra work only in the else branch — gas difference reveals which path ran
    suint256 tmp = 0s;
    for (uint256 i = 0; i < 10; i++) {
        tmp = tmp + 1s;
    }
    discount = tmp;
}
```

**Why it leaks:** The EVM execution trace shows which opcodes were executed. If the `if` branch runs different code (or a different amount of code) than the `else` branch, observers can determine which branch was taken, revealing the value of the shielded boolean.

**What to do instead:** Ensure both branches execute the same operations with the same gas cost. A ternary where both arms are simple assignments of the same type is safe — the EVM does the same work for both paths.

```solidity
// BETTER: Both arms are identical operations (single assignment), same gas either way
discount = isVIP ? 50s : 0s;
```

### Literals

**The problem:** Assigning literal values to shielded types — whether via explicit cast or the [`s` suffix](shielded-literals.md) — embeds those values directly in the contract bytecode, which is publicly visible.

```solidity
// Both forms embed `42` in bytecode
suint256 a = suint256(42);
suint256 b = 42s;
```

**What to do instead:** Be aware that literals are embedded in contract bytecode and are publicly visible. The compiler emits warning 9660 for all shielded literals to remind you. If the initial value is sensitive, introduce it via encrypted calldata instead of hardcoding it.

### Dynamic Loops

**The problem:** Using a shielded value as a loop bound leaks the value through gas consumption. Each iteration costs gas, so the total gas used reveals how many times the loop executed.

```solidity
// BAD: Leaks the value of `shieldedCount` via gas
suint256 shieldedCount = /* ... */;
for (uint256 i = 0; i < uint256(shieldedCount); i++) {
    // Each iteration is visible in gas cost
}
```

**Why it leaks:** Gas is publicly visible. If a loop runs 5 times vs. 100 times, the gas difference is observable. This reveals the shielded loop bound.

**What to do instead:** Use a fixed-size loop with a known maximum, and perform no-op iterations when the actual count is smaller. The no-op path must cost the same gas as the real-work path — otherwise an observer can count how many iterations did real work by comparing per-iteration gas costs.

```solidity
// BETTER: Fixed-size loop with constant iteration count
uint256 constant MAX_ITERATIONS = 100;
for (uint256 i = 0; i < MAX_ITERATIONS; i++) {
    // Use a shielded condition to decide whether to actually do work.
    // IMPORTANT: Both the "real work" and "no-op" paths must use
    // the same gas -- e.g., write to the same slots, do the same
    // number of arithmetic ops, etc.
}
```

### Unprotected View Functions

**The problem:** If you write a view function that unshields and returns private data without access control, anyone can read it. The shielded storage is meaningless if a public getter exposes the plaintext.

```solidity
// BAD: Anyone can call this and read the shielded value
suint256 private _secretBalance;

function secretBalance() external view returns (uint256) {
    return uint256(_secretBalance);
}
```

**Why it leaks:** The function casts the shielded value to a plain `uint256` and returns it with no restriction on who can call it. The value is returned in plaintext to the caller.

**What to do instead:** Always add access control to view functions that unshield data. If the getter checks `msg.sender`, callers must use a [signed read](../reference/rpc-methods/eth-call.md#signed-calls-signed-reads) — otherwise `msg.sender` will be the zero address.

```solidity
suint256 private _secretBalance;

function secretBalance() external view returns (uint256) {
    require(msg.sender == owner, "Not authorized");
    return uint256(_secretBalance);
}
```

{% hint style="info" %}
Access control doesn't have to be sender-based. Time-locked reveals, role-based access, or any other gating logic is fine — the important thing is that the function doesn't unconditionally return private data.
{% endhint %}

### Public Shielded Variables

**The problem:** Declaring a shielded variable as `public` will not compile. Solidity automatically generates a public getter for `public` state variables, which would return the shielded value -- violating the rule that shielded types cannot be returned from `public` or `external` functions.

```solidity
// Will NOT compile
suint256 public secretBalance;

// Will NOT compile
function getSecret() external view returns (suint256) {
    return secretBalance;
}
```

**What to do instead:** Declare shielded variables as `private` or `internal`. If you need to expose the value, unshield it explicitly with a cast, and use access control (see above).

### Unencrypted Calldata

**The problem:** There is currently nothing enforcing that functions with shielded parameters are called via a Seismic transaction (type `0x4A`). You can call a function that accepts `suint256` with a regular transaction, and the shielded parameter values will be visible in plaintext in the transaction's input data. Likewise, you can use a Seismic transaction to call a function with no shielded inputs — the encryption is unnecessary but harmless.

```solidity
// This function accepts shielded input, but nothing prevents calling it
// with a regular (non-Seismic) transaction — which would leak `amount`.
function deposit(suint256 amount) external {
    balances[msg.sender] += amount;
}
```

**What to do instead:** If a function has any shielded inputs, always call it via a Seismic transaction. This is the caller's responsibility — the contract cannot currently enforce it. We plan to tighten this up in the future so the runtime rejects non-Seismic calls to functions with shielded parameters.

### Exponentiation

**The problem:** The `**` operator has a gas cost that scales with the value of the exponent. If the exponent is a shielded value, the gas cost reveals the exponent.

```solidity
// BAD: Gas cost reveals the value of `shieldedExp`
suint256 base = 2s;
suint256 shieldedExp = /* ... */;
suint256 result = base ** shieldedExp;  // Gas cost leaks shieldedExp
```

**Why it leaks:** The EVM's modular exponentiation implementation uses more gas for larger exponents. An observer monitoring gas consumption can estimate the exponent value.

**What to do instead:** Avoid using shielded values as exponents entirely. If you need exponentiation with a private exponent, consider alternative algorithms and make sure you think carefully about their gas consumption profile.

### Enums

**The problem:** Enum types have a small, known range of possible values. If you convert an enum to a shielded type, the limited range makes it easy to guess the shielded value — an observer only needs to try a handful of possibilities.

```solidity
enum Status { Active, Inactive, Suspended }
// Only 3 possible values (0, 1, 2) — shielding provides little protection
suint256 shieldedStatus = suint256(uint256(Status.Active));
```

**What to do instead:** Consider whether the limited range of possible values undermines the privacy guarantee. Shielding an enum with 3 members is not meaningfully private.

### `immutable` and `constant` Shielded Variables

Shielded types **cannot** be declared as `immutable` or `constant`. The compiler will reject both with an error. Constants are embedded in bytecode (publicly visible), and immutables are stored in bytecode after construction — neither is compatible with the confidential storage model.

```solidity
// Will NOT compile — compiler error
suint256 immutable SECRET = 42s;

// Will NOT compile — compiler error
suint256 constant MY_VALUE = 1s;
```

**What to do instead:** Use a regular `private` shielded variable initialized in the constructor or via a setter function called through a Seismic transaction.

### RNG Proposer Bias

**The problem:** The synchronous [RNG precompile](../reference/precompiles/rng.md) produces randomness that is deterministic given the enclave's secret key, the transaction hash, remaining gas, and personalization bytes. In theory, a block proposer could simulate RNG outputs and selectively include, exclude, or reorder transactions to influence outcomes.

```solidity
// In theory, a proposer could simulate this output and decide
// whether to include the transaction based on the result.
function drawWinner() external {
    suint256 rand = rng256();
    uint256 winnerIndex = uint256(rand) % participants.length;
    winner = participants[winnerIndex];
}
```

**Why this matters:** Seismic's TEE setup largely mitigates this — proposers are restricted in what they can observe and do, and we believe synchronous RNG is safe for most use cases. This is something we've thought about extensively. However, for applications with especially high-stakes randomness requirements, it's worth being aware of the theoretical attack surface.

**What to do instead:** For the most sensitive randomness use cases (large-pot lotteries, leader elections), consider using an asynchronous commit-reveal scheme where entropy is committed before the block in which it is consumed. Seismic does not provide this out of the box today. For the vast majority of use cases, the synchronous RNG precompile is appropriate.
