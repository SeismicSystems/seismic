---
icon: wave-sine
---

# Opcodes

Seismic introduces two new EVM instructions to handle confidential storage: `CLOAD` and `CSTORE`. These work alongside the standard `SLOAD` and `SSTORE`, with access enforced through **FlaggedStorage** — a modified storage model where every slot carries an `is_private` flag.

## `CLOAD`

Reads a value from a storage slot. `CLOAD` can access both private and public slots — it is strictly more powerful than `SLOAD`. Note that the tradeoff is that `CLOAD` will consume more gas: if it didn't, it would leak information about the size of the shielded value

```solidity
contract PrivateBalance {
    suint256 private balance;

    function getBalance() external view returns (suint256) {
        return balance;
        // Compiler generates:
        //   CLOAD slot(balance) → returns the value (works for both private and public slots)
    }
}
```

Using inline assembly:

```solidity
function cloadExample(uint256 slot) internal view returns (uint256 value) {
    assembly {
        // 0xB0 = CLOAD opcode
        // Reads from `slot` only if is_private = true
        value := cload(slot)
    }
}
```

Opcode: `0xB0`

## `CSTORE`

Stores a value and marks the slot as **private** (confidential). The gas cost is the same for storing any value, regardless of size. This means `CSTORE` will be more expensive than `SSTORE` by design

```solidity
contract PrivateCounter {
    suint256 private count;  // shielded type → compiler uses CSTORE

    function increment() external {
        count = count + suint256(1);
        // Compiler generates:
        //   CLOAD  slot(count)       → read current value
        //   CSTORE slot(count), new  → write new value, slot marked is_private=true
    }
}
```

Using inline assembly:

```solidity
function cstoreExample(uint256 slot, uint256 value) internal {
    assembly {
        // 0xB1 = CSTORE opcode
        // Writes `value` to `slot` and sets is_private = true
        cstore(slot, value)
    }
}
```

Opcode: `0xB1`

## `SLOAD` / `SSTORE` (Standard)

Standard EVM storage operations. `SLOAD` reverts if the slot is private — unlike in Ethereum, `SLOAD` can have side effects. `SSTORE` marks the slot as **public**, and reverts if the slot is already marked private.

```solidity
contract PublicCounter {
    uint256 public count;  // regular type → compiler uses SLOAD/SSTORE

    function increment() external {
        count += 1;
        // Compiler generates:
        //   SLOAD  slot(count)       → read current value (reverts if private)
        //   SSTORE slot(count), new  → write new value, slot marked is_private=false
    }
}
```

## `TIMESTAMP_MS`

Returns the current block timestamp in **milliseconds**. Seismic produces multiple blocks per second, so standard `TIMESTAMP` (which returns unix seconds) cannot distinguish between them.

In Seismic Solidity:

* `block.timestamp` — returns unix seconds, same as standard Solidity
* `block.timestamp_ms` — returns unix milliseconds
* `block.timestamp_seconds` — alias for `block.timestamp`, for readability

```solidity
contract TimestampExample {
    function getTimestamps() external view returns (uint256 sec, uint256 ms) {
        sec = block.timestamp;       // unix seconds (same as Ethereum)
        ms = block.timestamp_ms;     // unix milliseconds (Seismic-specific)

        assert(block.timestamp_seconds == block.timestamp);
        assert(block.timestamp_ms / 1000 == block.timestamp);
    }
}
```

## FlaggedStorage Access Rules

Every storage slot is a pair: `(value: U256, is_private: bool)`. The flag is set by the first store operation and cannot be changed after that.

### Load

| Operation                        | Result            |
| -------------------------------- | ----------------- |
| `CLOAD` on a private slot        | Returns the value |
| `CLOAD` on a public slot         | Returns the value |
| `CLOAD` on an uninitialized slot | Returns zero      |
| `SLOAD` on a public slot         | Returns the value |
| `SLOAD` on a private slot        | Reverts           |
| `SLOAD` on an uninitialized slot | Returns zero      |

### Store

| Operation                         | Result                     |
| --------------------------------- | -------------------------- |
| `CSTORE` to a private slot        | Writes, stays private      |
| `CSTORE` to a public slot         | Reverts                    |
| `CSTORE` to an uninitialized slot | Writes, marks as private   |
| `SSTORE` to a public slot         | Writes, stays public       |
| `SSTORE` to a private slot        | Reverts                    |
| `SSTORE` to an uninitialized slot | Writes, marks as public    |

## Example: Mixed Public and Private Storage

```solidity
contract MixedStorage {
    uint256 public totalDeposits;    // public  → SLOAD/SSTORE
    suint256 private userBalance;    // private → CLOAD/CSTORE

    function deposit() external payable {
        // Public update — visible on-chain
        totalDeposits += msg.value;
        // Compiler: SLOAD slot(0) → SSTORE slot(0), is_private=false

        // Private update — hidden from observers
        userBalance = userBalance + suint256(msg.value);
        // Compiler: CLOAD slot(1) → CSTORE slot(1), is_private=true
    }
}
```

An external observer calling `eth_getStorageAt` can read `totalDeposits` (slot 0) but `userBalance` (slot 1) returns zero, as if the slot were uninitialized.

## Gas

`CLOAD` and `CSTORE` have **constant gas costs** regardless of the value being read or written. This prevents observers from inferring information about shielded values by analyzing gas consumption. See [Storage](../seismic-solidity/storage.md#gas-considerations) for details.

## Related

- [Storage](../seismic-solidity/storage.md) — FlaggedStorage model, slot packing, gas details
- [Shielded Types](../seismic-solidity/shielded-types.md) — `suint`, `sint`, `saddress`, `sbool`
- [eth_getStorageAt](rpc-methods/eth-get-storage-at.md) — RPC behavior for private slots
