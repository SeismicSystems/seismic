---
icon: shield-check
---

# Best Practices

These are guidelines for writing secure Seismic contracts. Following them will help you avoid the most common privacy mistakes.

### Always Use Seismic Transactions for Shielded Parameters

Any function that accepts shielded types as parameters should be called using a Seismic transaction (type `0x4A`), which encrypts the calldata. If calldata is not encrypted, the shielded parameter values are visible in plaintext in the transaction's input data, defeating the purpose of using shielded types. This applies to all function calls that pass shielded values as parameters.

Your client library (e.g., `seismic-viem`) handles Seismic transaction construction automatically when you use shielded write functions.

```solidity
// This function should ONLY be called via a Seismic transaction
function deposit(suint256 amount) external {
    balances[msg.sender] += amount;
}
```

### Be Mindful of Gas-Based Information Leakage

Gas consumption is publicly visible. Any operation whose gas cost depends on a shielded value is a potential leak. The main offenders are:

- **Conditional branches** on shielded booleans (different branches use different gas).
- **Loops** with shielded bounds (iteration count is visible via gas).
- **Exponentiation** with shielded exponents (gas scales with exponent value).

Use constant-time patterns: fixed-size loops, branchless logic, and avoid shielded exponents.

`CLOAD` and `CSTORE` themselves have constant gas cost by design -- the risk comes from higher-level patterns.

### Use Casting Carefully

Every cast between shielded and unshielded types is a potential information leak. See [Casting](casting.md) for details.

- **Unshielded to shielded**: The input value is visible in the trace.
- **Shielded to unshielded**: The output value is visible in the trace.

Minimize casts. During security review, identify every cast and confirm the exposure is intentional.

### Review Compiler Warnings

The Seismic Solidity compiler generates a ton of warnings specific to shielded types. This is designed to be annoying because of how easy it is to make a mistake. Please review each warning carefully. These warnings flag potential privacy issues such as:

- Shielded values used in contexts that may leak information.
- Casts that expose confidential data.
- Patterns known to be risky.

Do not ignore these warnings. Treat them as seriously as you would treat security audit findings.

### Test with `sforge test`

Use `sforge test` (the Seismic fork of Foundry's `forge test`) to run your test suite. It supports shielded types natively and can catch issues specific to confidential computation that standard `forge test` would miss.

```bash
sforge test
```

Write tests that specifically verify:

- Shielded values remain confidential through the expected code paths.
- Access control prevents unauthorized reads of unshielded data.

### Keep Shielded Data in the Shielded Domain

The longer a value stays shielded, the more private it is. Avoid unnecessary round-trips between shielded and unshielded types. Perform as much computation as possible in the shielded domain before unshielding a final result (if unshielding is even needed at all).

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
