---
hidden: true
icon: face-hand-yawn
---

# Gotchas

* conditional execution
* dynamic loops

#### 3.3 Literals and Enums

* Be cautious when using literals and enums with shielded types. They can inadvertently leak information if not handled properly.



#### 3.4 Exponentiation and Gas Costs

* Using shielded integers as exponents in exponentiation operations can leak information through gas usage, as gas cost scales with the exponent value.



#### 3.5 `.min()` and `.max()` Functions

* Calling `.min()` and `.max()` on shielded integers can reveal information about their values.



#### 3.6 `immutable` Variables

* Shielded `immutable` variables are only truly confidential if the transaction calldata used during their instantiation is encrypted.



* **Avoid Public Exposure**: Never expose shielded variables through public getters or events.
* **Careful with Gas Usage**: Be mindful of operations where gas cost can vary based on shielded values (e.g., loops, exponentiation).
* **Encrypt Calldata**: Ensure that any calldata used for initializing shielded `immutable` variables is encrypted.
* **Manual Slot Packing**: If slot packing is necessary, use inline assembly carefully to avoid introducing vulnerabilities.
* **Review Compiler Warnings**: Pay attention to compiler warnings related to shielded types to prevent accidental leaks.
