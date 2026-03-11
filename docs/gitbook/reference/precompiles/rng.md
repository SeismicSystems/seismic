---
icon: dice
---

# RNG (`0x64`)

Securely generate random bytes inside the TEE. The randomness is derived from TEE-internal entropy combined with optional personalization data, ensuring that the output is unpredictable to all parties -- including the node operator.

## Input

Raw bytes in the following layout:

| Offset  | Field              | Type     | Description                                              |
| ------- | ------------------ | -------- | -------------------------------------------------------- |
| `[0:4]` | output length      | `uint32` | Number of random bytes to generate (big-endian)          |
| `[4:]`  | personalization    | `bytes`  | Optional personalization data to influence the RNG output |

## Output

| Bytes        | Type    | Description                                     |
| ------------ | ------- | ----------------------------------------------- |
| random bytes | `bytes` | Securely generated random bytes of the requested length |

{% hint style="info" %}
**Built-in helpers.** Seismic Solidity provides built-in functions that call this precompile and automatically cast the result to the appropriate shielded type:

* **Shielded integers:** `sync_rng8()` → `suint8`, `sync_rng16()` → `suint16`, `sync_rng32()` → `suint32`, `sync_rng64()` → `suint64`, `sync_rng128()` → `suint128`, `sync_rng256()` → `suint256`
* **Shielded fixed bytes:** `sync_rng_b1()` → `sbytes1`, `sync_rng_b2()` → `sbytes2`, ... `sync_rng_b32()` → `sbytes32`

If you use these helpers, you don't need to deal with the raw precompile interface or manual casting. See the [examples below](#without-personalization).
{% endhint %}

{% hint style="warning" %}
**A note on proposer bias.** The RNG precompile produces randomness that is a deterministic function of the enclave's secret key, the transaction hash, remaining gas, and any personalization bytes you provide. In theory, a block proposer could simulate RNG outputs for candidate transactions and selectively include, exclude, or reorder them to influence randomness-dependent outcomes.

In practice, this is largely mitigated by Seismic's TEE setup — proposers are restricted in what they can observe and do. This is something we've thought about extensively, and we believe synchronous RNG is safe for most use cases. That said, if your application has especially high-stakes randomness requirements (e.g. large-pot lotteries, leader elections), consider using an asynchronous commit-reveal scheme where entropy is committed before the block in which it is consumed. Seismic does not provide this out of the box today.

See also: [Footguns — RNG proposer bias](../../seismic-solidity/footguns.md#rng-proposer-bias).
{% endhint %}

## Use cases

* Shuffling hidden card decks or secret orderings
* Generating nonces for on-chain cryptographic operations

## Without personalization

Seismic Solidity provides built-in helper functions to call the synchronous RNG precompile without personalization bytes: `sync_rng8()`, `sync_rng16()`, `sync_rng32()`, `sync_rng64()`, `sync_rng128()`, `sync_rng256()`, and `sync_rng_b1()` through `sync_rng_b32()`. These return the corresponding shielded type (`suint8`, `suint256`, `sbytes1`, etc.).

```solidity
suint256 random = sync_rng256();
```

## With personalization

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
