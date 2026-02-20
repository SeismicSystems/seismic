---
icon: right-left
---

# Casting

## Explicit Casting Only

Shielded types and their unshielded counterparts do **not** support implicit casting. You must always cast explicitly. This is a deliberate design decision -- every conversion between shielded and unshielded types is a potential information boundary, and the compiler requires you to be explicit about crossing it.

```solidity
uint256 publicNumber = 100;

// Implicit casting -- will NOT compile
suint256 shielded = publicNumber; // Error

// Explicit casting -- correct
suint256 shielded = suint256(publicNumber); // OK
```

This applies to all shielded types: `suint`, `sint`, `sbool`, and `saddress`.

```solidity
bool flag = true;
sbool shieldedFlag = sbool(flag);          // OK

address addr = msg.sender;
saddress shieldedAddr = saddress(addr);    // OK

int256 signed = -42;
sint256 shieldedSigned = sint256(signed);  // OK
```

## Shielding Values (Unshielded to Shielded)

When you cast from an unshielded type to a shielded type, you are "shielding" the value -- moving it from the public domain into confidential storage/computation.

```solidity
uint256 publicValue = 100;
suint256 shieldedValue = suint256(publicValue);
```

{% hint style="warning" %}
**Privacy consideration:** When going from unshielded to shielded, the original unshielded value is visible in the transaction trace at the point of casting. Observers can see what value was shielded. The value only becomes confidential after the cast, in subsequent operations.
{% endhint %}

If you need the value to be confidential from the start, it should arrive as encrypted calldata via a Seismic transaction (type `0x4A`), not be cast from a public variable.

## Unshielding Values (Shielded to Unshielded)

When you cast from a shielded type to an unshielded type, you are "unshielding" the value -- making it publicly visible.

```solidity
suint256 shieldedValue = /* ... */;
uint256 publicValue = uint256(shieldedValue);
```

{% hint style="warning" %}
**Privacy consideration:** When going from shielded to unshielded, the final unshielded value is visible in the transaction trace. Observers can see the result. This permanently exposes the value.
{% endhint %}

This is sometimes necessary (e.g., returning a value from a view function or interfacing with a non-shielded contract), but you should be deliberate about when and why you do it.

## Casting `saddress` to `payable`

To cast an `saddress` to a `payable` address, use the following pattern:

```solidity
address payable pay = payable(address(saddress(someSaddressValue)));
```

You can also go in the other direction:

```solidity
address payable pay = /* ... */;
saddress shielded = saddress(address(pay));
```

Note that `saddress payable` does not exist as a type. If you need to send ETH to a shielded address, you must first unshield it to a regular `address payable`, which will expose the address.

## Size Casting Between Shielded Integers

You can cast between different sizes of shielded integers, just as you can with regular Solidity integers:

```solidity
suint128 smaller = suint128(42);
suint256 larger = suint256(smaller);  // Widening: safe, no data loss

suint256 big = suint256(1000);
suint128 small = suint128(big);       // Narrowing: may truncate
```

The same rules that apply to regular Solidity integer casting apply here:

* **Widening** (smaller to larger): Always safe, the value is preserved.
* **Narrowing** (larger to smaller): May truncate the value if it exceeds the target type's range.

You can also cast between signed and unsigned shielded integers:

```solidity
suint256 unsigned = suint256(100);
sint256 signed = sint256(unsigned);   // Reinterprets the bits
```

## Common Patterns

### Returning values from view functions

Since shielded types cannot be returned from `public` or `external` functions, you must unshield them first:

```solidity
suint256 private balance;

// Will NOT compile -- cannot return shielded type from external function
function getBalance() external view returns (suint256) { ... }

// Correct -- unshield before returning
function getBalance() external view returns (uint256) {
    return uint256(balance);
}
```

{% hint style="info" %}
Returning an unshielded value from a view function makes it visible to the caller. If the caller should only see their own data, use access control and [signed reads](/broken/pages/Iy1iEnGF6LoJvAy1G6ix) to ensure only authorized users can query it.
{% endhint %}

### Interfacing with non-shielded contracts

When calling a contract that expects unshielded types, cast at the call boundary:

```solidity
suint256 private amount;

function sendToExternal(address externalContract) external {
    uint256 unshielded = uint256(amount);
    IExternalContract(externalContract).deposit(unshielded);
}
```

### Shielding input from encrypted calldata

The most private way to introduce a value is through encrypted calldata, where the value is never visible in plaintext on-chain:

```solidity
// The `amount` parameter arrives encrypted via a Seismic transaction (0x4A)
// and is decrypted inside the TEE -- it is never publicly visible.
function deposit(suint256 amount) external {
    balances[msg.sender] += amount;
}
```

## Security Implications

Every cast between shielded and unshielded types is a potential information leak point. Keep these principles in mind:

1. **Unshielded-to-shielded casts** expose the input value in the trace. If the value was meant to be secret from the start, use encrypted calldata instead.
2. **Shielded-to-unshielded casts** expose the output value in the trace. Only unshield when you intend the value to become public.
3. **Minimize casts.** The fewer times you cross the shielded/unshielded boundary, the smaller your attack surface.
4. **Audit cast points.** During security review, identify every cast in your contract and verify that the information exposure at each point is intentional and acceptable.

```solidity
// Example: Unintended leak through intermediate cast
suint256 private secretA;
suint256 private secretB;

function leakyCompare() external view returns (bool) {
    // BAD: Both secrets are unshielded just to compare them publicly
    return uint256(secretA) > uint256(secretB);
}

function safeCompare() external view returns (sbool) {
    // BETTER: Compare in the shielded domain, no information leaves
    return secretA > secretB;
    // Note: sbool cannot be returned from external -- this is illustrative.
    // In practice, you would use the comparison result internally.
}
```
