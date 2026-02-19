---
description: >-
  Step-by-step guide to migrating Ethereum contracts and dApps to Seismic
icon: route
---

# Migrating from Ethereum

## Overview

If you have an existing Ethereum contract, migrating to Seismic is straightforward. Standard Solidity works unchanged on Seismic -- the EVM is a superset of Ethereum's. You only need to modify the parts of your contract and client code that you want to make private.

This guide walks through each step, from installing the Seismic toolchain to deploying on the network.

---

## Step 1: Install Seismic tools

The Seismic development suite replaces Foundry's tools with privacy-aware equivalents:

| Ethereum tool | Seismic equivalent | Purpose                |
| ------------- | ------------------ | ---------------------- |
| `forge`       | `sforge`           | Testing and building   |
| `anvil`       | `sanvil`           | Local development node |
| `solc`        | `ssolc`            | Solidity compiler      |

Install the Seismic toolchain by following the [Installation](../getting-started/installation.md) guide. The short version:

```bash
# Install sfoundryup
curl -L \
     -H "Accept: application/vnd.github.v3.raw" \
     "https://api.github.com/repos/SeismicSystems/seismic-foundry/contents/sfoundryup/install?ref=seismic" | bash
source ~/.zshenv  # or ~/.bashrc

# Install sforge, sanvil, ssolc
sfoundryup
source ~/.zshenv  # or ~/.bashrc
```

---

## Step 2: Decide what to shield

Not everything needs to be private. Shielding adds complexity to client interactions (signed reads, encrypted events), so you should be intentional about what you protect.

**Good candidates for shielding:**

- User balances
- Transaction amounts
- Positions (in games, orderbooks, etc.)
- Votes and ballot choices
- Personal data (addresses, identifiers)

**Usually fine to leave public:**

- Total supply / aggregate counters
- Contract metadata (name, symbol, decimals)
- Immutable configuration
- Admin addresses
- Timestamps and block-related data

---

## Step 3: Update types

For every state variable you want to shield, change the type from its standard Solidity equivalent to the shielded version:

| Standard type          | Shielded type            |
| ---------------------- | ------------------------ |
| `uint256`              | `suint256`               |
| `uint128`, `uint64`... | `suint128`, `suint64`... |
| `int256`               | `sint256`                |
| `address`              | `saddress`               |
| `bool`                 | `sbool`                  |

**Before (Ethereum):**

```solidity
mapping(address => uint256) public balanceOf;
uint256 public totalSupply;
```

**After (Seismic):**

```solidity
mapping(address => suint256) private balanceOf;  // shielded
uint256 public totalSupply;                      // stays public
```

The compiler handles the rest. Shielded types compile to `CLOAD`/`CSTORE` instead of `SLOAD`/`SSTORE`. Arithmetic, comparisons, and assignments work exactly the same way.

For details on each shielded type, see [Shielded Types](../seismic-solidity/shielded-types/README.md).

---

## Step 4: Handle public getters

Solidity automatically generates getter functions for `public` state variables. When you change a variable to a shielded type, you must:

1. **Remove the `public` visibility** from the state variable (use `private` or `internal`).
2. **Add an explicit getter function** with access control, so only authorized callers can read the value.

**Before (Ethereum):**

```solidity
mapping(address => uint256) public balanceOf;
// Solidity auto-generates: function balanceOf(address) view returns (uint256)
```

**After (Seismic):**

```solidity
mapping(address => suint256) private balanceOf;

function getBalance(address account) external view returns (suint256) {
    require(msg.sender == account, "Only the owner can view their balance");
    return balanceOf[account];
}
```

{% hint style="info" %}
Callers must use a **signed read** to call getter functions that check `msg.sender`. A standard `eth_call` zeros out the `from` field on Seismic, so `msg.sender` would be `address(0)`. Client libraries like `seismic-viem` handle signed reads automatically. See [Signed Reads](../reference/seismic-transaction/signed-reads.md) for details.
{% endhint %}

---

## Step 5: Update events

Shielded types cannot be emitted directly in events. The compiler will reject this:

```solidity
event Transfer(address from, address to, suint256 amount); // Will not compile
```

Instead, encrypt the sensitive parameters using the AES-GCM precompiles and emit the encrypted bytes:

```solidity
event EncryptedTransfer(
    address indexed from,
    address indexed to,
    bytes encryptedAmount
);

function transfer(address to, suint256 amount) external {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;

    // Encrypt the amount for the recipient using ECDH + HKDF + AES-GCM
    bytes memory encrypted = _encryptForRecipient(to, abi.encode(amount));
    emit EncryptedTransfer(msg.sender, to, encrypted);
}
```

For a complete walkthrough of the encrypted events pattern, see [Events](../seismic-solidity/events.md).

---

## Step 6: Update client code

Replace your Ethereum client libraries with their Seismic equivalents:

| Ethereum library | Seismic equivalent | Install                            |
| ---------------- | ------------------ | ---------------------------------- |
| `viem`           | `seismic-viem`     | `npm install seismic-viem`         |
| `wagmi`          | `seismic-react`    | `npm install seismic-react`        |
| `alloy` (Rust)   | `seismic-alloy`    | Add `seismic-alloy-provider` crate |
| `web3.py`        | `seismic-python`   | `pip install seismic-py`           |

Key changes in client code:

- Use **shielded wallet clients** instead of standard wallet clients. These handle Seismic transaction construction (type `0x4A`) and calldata encryption automatically.
- Use **signed reads** instead of plain `eth_call` when reading shielded data that checks `msg.sender`.
- Listen for encrypted events and decrypt them client-side using the shared secret derived from ECDH.

For library-specific guides, see [Client Libraries](../client-libraries/overview.md).

---

## Step 7: Test

Run your tests with `sforge` instead of `forge`. The test framework works the same way -- `sforge test` compiles with `ssolc` and runs against a local Seismic EVM.

```bash
sforge test
```

If you use a local node for integration tests, use `sanvil` instead of `anvil`:

```bash
sanvil
```

All standard Foundry testing patterns (fuzz testing, forking, cheatcodes) work with the Seismic tools.

---

## Step 8: Deploy

Deploy to the Seismic network using the same flow you would use for Ethereum, just with a different RPC URL and the Seismic tools.

**Deploy to devnet:**

```bash
sforge create src/MyContract.sol:MyContract \
    --rpc-url https://node-2.seismicdev.net/rpc \
    --private-key $PRIVATE_KEY
```

**Deploy with a script:**

```bash
sforge script script/Deploy.s.sol \
    --rpc-url https://node-2.seismicdev.net/rpc \
    --private-key $PRIVATE_KEY \
    --broadcast
```

For network details (RPC URLs, chain IDs, faucets), see [Devnet](devnet.md) and [Testnet](testnet.md).

---

## Common pitfalls

### Forgetting to use Seismic transactions for shielded writes

If you send a standard Ethereum transaction to a function that writes shielded data, the calldata is visible in plaintext on-chain. Always use a Seismic transaction (type `0x4A`) when the calldata contains sensitive arguments. Client libraries handle this automatically when you use the shielded wallet client.

### Using public getters on shielded types

The Solidity compiler will not let you declare a `public` state variable with a shielded type. But even if you write an explicit getter that returns a shielded value, remember that callers need signed reads to access it with a valid `msg.sender`.

### Not encrypting events

Emitting shielded values in events requires manual encryption via the AES-GCM precompiles. If you cast a shielded value to its unshielded counterpart and emit it directly, the value becomes permanently visible in the transaction logs.

### Mixing shielded and unshielded types carelessly

Casting a shielded value to an unshielded type (e.g., `uint256(myShieldedValue)`) makes it visible in the execution trace. Only cast when you intentionally want to make a value public.

### Not updating the frontend

If you shield contract state but continue using a standard viem client, your transactions will not encrypt calldata and your reads will not be signed. Update to `seismic-viem` or `seismic-react` as described in Step 6.
