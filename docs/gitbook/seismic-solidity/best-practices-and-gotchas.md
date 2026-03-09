---
icon: triangle-exclamation
---

# Best Practices & Gotchas

This page covers the most common information leak vectors when working with shielded types, followed by best practices for writing secure Seismic contracts. Understanding these gotchas is essential -- shielded types protect values at rest and in transit, but careless usage patterns can leak information through side channels.

## Part 1: Gotchas (Information Leak Vectors)

### 1. Conditional Execution

**The problem:** Using an `sbool` in a conditional branch leaks information through the execution trace and gas consumption. Observers can tell which branch was taken by examining the gas used or the operations performed.

```solidity
// BAD: Leaks the value of `isVIP` via execution trace
sbool isVIP = /* ... */;
if (isVIP) {
    discount = suint256(50);  // This branch uses different gas than the else branch
} else {
    discount = suint256(0);
}
```

**Why it leaks:** The EVM execution trace shows which opcodes were executed. If the `if` branch runs different code (or a different amount of code) than the `else` branch, observers can determine which branch was taken, revealing the value of the shielded boolean.

**What to do instead:** Use branchless patterns where both paths execute the same operations regardless of the condition. Structure your logic so that the same code runs in both cases, with only the shielded values differing.

```solidity
// BETTER: Branchless pattern -- same operations regardless of condition
// Both multiplications execute, observer cannot distinguish the path
discount = isVIP ? suint256(50) : suint256(0);
```

### 2. Dynamic Loops

**The problem:** Using a shielded value as a loop bound leaks the value through gas consumption. Each iteration costs gas, so the total gas used reveals how many times the loop executed.

```solidity
// BAD: Leaks the value of `shieldedCount` via gas
suint256 shieldedCount = /* ... */;
for (uint256 i = 0; i < uint256(shieldedCount); i++) {
    // Each iteration is visible in gas cost
}
```

**Why it leaks:** Gas is publicly visible. If a loop runs 5 times vs. 100 times, the gas difference is observable. This reveals the shielded loop bound.

**What to do instead:** Use a fixed-size loop with a known maximum, and perform no-op iterations when the actual count is smaller. This way, every invocation uses the same gas regardless of the shielded value.

```solidity
// BETTER: Fixed-size loop with constant iteration count
uint256 constant MAX_ITERATIONS = 100;
for (uint256 i = 0; i < MAX_ITERATIONS; i++) {
    // Use a shielded condition to decide whether to actually do work
    // Both the "real work" and "no-op" paths should cost the same gas
}
```

### 3. Exponentiation

**The problem:** The `**` operator has a gas cost that scales with the value of the exponent. If the exponent is a shielded value, the gas cost reveals the exponent.

```solidity
// BAD: Gas cost reveals the value of `shieldedExp`
suint256 base = suint256(2);
suint256 shieldedExp = /* ... */;
suint256 result = base ** shieldedExp;  // Gas cost leaks shieldedExp
```

**Why it leaks:** The EVM's modular exponentiation implementation uses more gas for larger exponents. An observer monitoring gas consumption can estimate the exponent value.

**What to do instead:** Avoid using shielded values as exponents entirely. If you need exponentiation with a private exponent, consider alternative algorithms or precomputed lookup tables.

### 4. Literals and Enums

**The problem:** Assigning literal values or enum members to shielded types can inadvertently reveal information. The literal value itself is visible in the bytecode, and when it interacts with shielded values, it may narrow the range of possible values an observer needs to consider.

```solidity
// Potentially problematic: The literal `42` is visible in bytecode
suint256 shielded = suint256(42);

// Enum values are also visible in bytecode
enum Status { Active, Inactive, Suspended }
// If you convert an enum to a shielded type, the small range of possible
// values (0, 1, 2) makes it easier to guess the shielded value.
```

**What to do instead:** Be aware that literals are embedded in contract bytecode and are publicly visible. When working with small-range values (like enums), consider whether the limited range of possible values undermines the privacy guarantee.

### 5. `.min()` and `.max()` Functions

**The problem:** Calling `.min()` and `.max()` on shielded integers can reveal information about the values being compared.

```solidity
// BAD: Can reveal information about the relative ordering of values
suint256 a = /* ... */;
suint256 b = /* ... */;
suint256 smaller = a.min(b);  // Reveals which is smaller
```

**Why it leaks:** The result of `.min()` or `.max()` is one of the two input values. If the result is later unshielded or used in a context where it becomes observable, the ordering relationship between the two inputs is revealed.

**What to do instead:** If you need to use the minimum or maximum of two shielded values, keep the result shielded and avoid exposing it. Be cautious about how the result flows through your contract.

### 6. `immutable` Variables

**The problem:** Shielded `immutable` variables are only truly confidential if the constructor calldata is encrypted. If the contract is deployed via a regular (non-Seismic) transaction, the constructor arguments are visible in plaintext.

```solidity
contract MyContract {
    suint256 immutable SECRET;

    constructor(suint256 _secret) {
        SECRET = _secret;  // Only private if constructor calldata is encrypted
    }
}
```

**Why it leaks:** `immutable` variables are set during construction and stored in the contract bytecode. If the constructor calldata is not encrypted, the initial value is visible in the deployment transaction.

**What to do instead:** Always deploy contracts with shielded `immutable` variables using a Seismic transaction (type `0x4A`), which encrypts the calldata. If you cannot guarantee encrypted deployment, do not use shielded `immutable` variables for sensitive data.

### 7. Public Getters

**The problem:** Declaring a shielded variable as `public` will not compile. Solidity automatically generates a public getter for `public` state variables, which would return the shielded value -- violating the rule that shielded types cannot be returned from `public` or `external` functions.

```solidity
// Will NOT compile
suint256 public secretBalance;

// Will NOT compile
function getSecret() external view returns (suint256) {
    return secretBalance;
}
```

**What to do instead:** Declare shielded variables as `private` or `internal`. If you need to expose the value, unshield it explicitly with a cast, and use access control to limit who can read it.

```solidity
suint256 private secretBalance;

function getBalance() external view returns (uint256) {
    // Access control should be applied here
    return uint256(secretBalance);
}
```

### 8. Constants

**The problem:** Shielded types cannot be declared as `constant`. Constants are embedded directly in the contract bytecode, which is publicly accessible. A "shielded constant" is a contradiction -- the value would be visible to anyone who reads the bytecode.

```solidity
// Will NOT compile
suint256 constant MY_SECRET = 42;
```

**What to do instead:** Use a regular `private` shielded variable initialized in the constructor or via a Seismic transaction. If the value truly does not need to be secret (e.g., it is a protocol parameter), use a regular unshielded `constant` instead.

---

## Part 2: Best Practices

### 1. Always Encrypt Calldata for Shielded Parameters

Any function that accepts shielded types as parameters should be called using a Seismic transaction (type `0x4A`), which encrypts the calldata. If calldata is not encrypted, the shielded parameter values are visible in plaintext in the transaction's input data, defeating the purpose of using shielded types.

This applies to:

- Constructor arguments for contracts with shielded `immutable` variables.
- All function calls that pass shielded values as parameters.

```solidity
// This function should ONLY be called via a Seismic transaction
function deposit(suint256 amount) external {
    balances[msg.sender] += amount;
}
```

### 2. Be Mindful of Gas-Based Information Leakage

Gas consumption is publicly visible. Any operation whose gas cost depends on a shielded value is a potential leak. The main offenders are:

- **Conditional branches** on shielded booleans (different branches use different gas).
- **Loops** with shielded bounds (iteration count is visible via gas).
- **Exponentiation** with shielded exponents (gas scales with exponent value).

Use constant-time patterns: fixed-size loops, branchless logic, and avoid shielded exponents.

`CLOAD` and `CSTORE` themselves have constant gas cost by design -- the risk comes from higher-level patterns.

### 3. Use Explicit Casting Deliberately

Every cast between shielded and unshielded types is a potential information leak. See [Casting](casting.md) for details.

- **Unshielded to shielded**: The input value is visible in the trace.
- **Shielded to unshielded**: The output value is visible in the trace.

Minimize casts. During security review, identify every cast and confirm the exposure is intentional.

### 4. Review Compiler Warnings

The Seismic Solidity compiler generates warnings specific to shielded types. These warnings flag potential privacy issues such as:

- Shielded values used in contexts that may leak information.
- Casts that expose confidential data.
- Patterns known to be risky.

Do not ignore these warnings. Treat them as seriously as you would treat security audit findings.

### 5. Use Seismic Transactions (Type `0x4A`)

For any transaction that involves shielded data -- whether sending, receiving, or computing on it -- use the Seismic transaction type (`0x4A`). Regular Ethereum transactions do not encrypt calldata, which means shielded parameter values would be visible in plaintext.

Your client library (e.g., `seismic-viem`) handles Seismic transaction construction automatically when you use shielded write functions.

### 6. Test with `sforge test`

Use `sforge test` (the Seismic fork of Foundry's `forge test`) to run your test suite. It supports shielded types natively and can catch issues specific to confidential computation that standard `forge test` would miss.

```bash
sforge test
```

Write tests that specifically verify:

- Shielded values remain confidential through the expected code paths.
- Casts happen only where intended.
- Access control prevents unauthorized reads of unshielded data.

### 7. Keep Shielded Data in the Shielded Domain

The longer a value stays shielded, the more private it is. Avoid unnecessary round-trips between shielded and unshielded types. Perform as much computation as possible in the shielded domain before unshielding a final result (if unshielding is needed at all).

```solidity
// BAD: Unnecessary unshielding and re-shielding
suint256 a = /* ... */;
suint256 b = /* ... */;
uint256 temp = uint256(a) + uint256(b);  // Both values leaked in trace
suint256 result = suint256(temp);

// GOOD: Stay in the shielded domain
suint256 a = /* ... */;
suint256 b = /* ... */;
suint256 result = a + b;  // No values leaked
```
