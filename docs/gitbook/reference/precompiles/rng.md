---
icon: dice
---

# RNG (`0x64`)

Synchronous random number generation inside the TEE. The randomness is derived from TEE-internal entropy combined with optional personalization data, ensuring that the output is unpredictable to all parties — including the node operator.

{% hint style="warning" %}
**Proposer bias.** A block proposer could theoretically simulate RNG outputs and selectively order transactions to influence outcomes. Seismic's TEE setup largely mitigates this, but high-stakes applications should consider commit-reveal schemes. See [Footguns — RNG Proposer Bias](../../seismic-solidity/footguns.md#rng-proposer-bias) for details.
{% endhint %}

## Input

Raw bytes in the following layout:

| Offset  | Field           | Type              | Description                                |
| ------- | --------------- | ----------------- | ------------------------------------------ |
| `[0:4]` | output length   | 4 bytes (uint32)  | Number of random bytes to generate         |
| `[4:]`  | personalization | arbitrary bytes   | Optional data to influence the RNG output  |

## Output

| Bytes        | Type    | Description                                     |
| ------------ | ------- | ----------------------------------------------- |
| random bytes | bytes | Random bytes of the requested length |

## Built-in helpers

Seismic Solidity provides built-in functions that call this precompile and automatically cast the result to the appropriate shielded type. If you use these, you don't need to deal with the raw precompile interface or manual casting.

**Shielded integers:**

* `unsafe_rng_u8()` → `suint8`
* `unsafe_rng_u16()` → `suint16`
* `unsafe_rng_u32()` → `suint32`
* `unsafe_rng_u64()` → `suint64`
* `unsafe_rng_u96()` → `suint96`
* `unsafe_rng_u128()` → `suint128`
* `unsafe_rng_u256()` → `suint256`

**Shielded fixed bytes:**

* `unsafe_rng_b1()` → `sbytes1`
* ...
* `unsafe_rng_b32()` → `sbytes32`

## Use cases

* Shuffling hidden card decks or secret orderings
* Generating nonces for on-chain cryptographic operations

## Examples

### Without personalization

```solidity
suint256 random = unsafe_rng_u256();
```

### With personalization

To pass personalization bytes, call the precompile directly at `0x64`:

```solidity
function getRandomBytesWithPersonalization(
    uint32 numBytes,
    bytes memory pers
) internal view returns (bytes memory) {
    // Without personalization, use abi.encodePacked(numBytes) instead
    (bool success, bytes memory result) = address(0x64).staticcall(
        abi.encodePacked(numBytes, pers)
    );
    require(success, "RNG precompile failed");
    return result;
}
```
