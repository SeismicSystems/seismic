---
description: CLAUDE.md template for Seismic smart contract development
icon: file-shield
---

# Seismic Solidity

Use this template when your project involves writing, testing, or deploying Solidity smart contracts on Seismic. It covers shielded types, the Seismic compiler toolchain, precompiles, and common contract patterns.

## The template

Copy the entire block below and save it as `CLAUDE.md` in your project root.

````markdown
# [Your Project Name]

## Seismic Overview

Seismic is an EVM-compatible L1 with on-chain privacy. Nodes run inside TEEs (Intel TDX). The Solidity compiler (`ssolc`) adds shielded types that compile to privacy-aware opcodes (`CLOAD`/`CSTORE`). Standard Solidity works unchanged — you only modify the parts you want to make private.

## Key Concepts

- **Shielded types**: `suint256`, `sint256`, `saddress`, `sbool` — stored with `CSTORE`, read with `CLOAD`. Values are invisible outside the TEE.
- **FlaggedStorage**: Each storage slot is `(value, is_private)`. Shielded types set the flag automatically.
- **TxSeismic (type 0x4A)**: Transaction type that encrypts calldata before broadcast. Client libraries handle this automatically.
- **Signed reads**: `eth_call` zeroes `msg.sender` on Seismic. To call view functions that check `msg.sender`, clients must use a signed read (type 0x4A call).
- **Precompiles**: Six cryptographic precompiles at addresses `0x64`–`0x69` (RNG, ECDH, AES-GCM encrypt/decrypt, HKDF, secp256k1 sign).

## Seismic Toolchain

The Seismic development tools are forks of Foundry:

| Tool     | Replaces | Install                    |
| -------- | -------- | -------------------------- |
| `ssolc`  | `solc`   | Installed via `sfoundryup` |
| `sforge` | `forge`  | Installed via `sfoundryup` |
| `sanvil` | `anvil`  | Installed via `sfoundryup` |

Install:

```bash
curl -L \
     -H "Accept: application/vnd.github.v3.raw" \
     "https://api.github.com/repos/SeismicSystems/seismic-foundry/contents/sfoundryup/install?ref=seismic" | bash
source ~/.zshenv  # or ~/.bashrc
sfoundryup
source ~/.zshenv  # or ~/.bashrc
```

## Core Patterns

### Shielded state variables

```solidity
// Private balance — stored with CSTORE, read with CLOAD
mapping(address => suint256) private balanceOf;

// Public total — standard SLOAD/SSTORE
uint256 public totalSupply;
```

### Access-controlled getter with signed read

```solidity
function getBalance(address account) external view returns (suint256) {
    require(msg.sender == account, "Only owner can view balance");
    return balanceOf[account];
}
// Callers MUST use a signed read — plain eth_call zeroes msg.sender
```

### Encrypted events (ECDH + HKDF + AES-GCM)

```solidity
event EncryptedTransfer(address indexed from, address indexed to, bytes encryptedAmount);

function transfer(address to, suint256 amount) external {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;

    // 1. ECDH shared secret
    (bool ok1, bytes memory secret) = address(0x65).staticcall(
        abi.encode(recipientPubKey, contractPrivKey)
    );
    // 2. Derive AES key
    (bool ok2, bytes memory aesKey) = address(0x68).staticcall(
        abi.encode(secret, "", "encryption-key", 32)
    );
    // 3. Encrypt
    (bool ok3, bytes memory encrypted) = address(0x66).staticcall(
        abi.encode(aesKey, nonce, abi.encode(amount), "")
    );

    emit EncryptedTransfer(msg.sender, to, encrypted);
}
```

### Using precompiles

| Address | Name            | Use                                                                 |
| ------- | --------------- | ------------------------------------------------------------------- |
| `0x64`  | RNG             | `address(0x64).staticcall(abi.encode(seed))`                        |
| `0x65`  | ECDH            | `address(0x65).staticcall(abi.encode(pubKey, privKey))`             |
| `0x66`  | AES-GCM Encrypt | `address(0x66).staticcall(abi.encode(key, nonce, plaintext, aad))`  |
| `0x67`  | AES-GCM Decrypt | `address(0x67).staticcall(abi.encode(key, nonce, ciphertext, aad))` |
| `0x68`  | HKDF            | `address(0x68).staticcall(abi.encode(ikm, salt, info, length))`     |
| `0x69`  | secp256k1 Sign  | `address(0x69).staticcall(abi.encode(hash, privKey))`               |

### Testing

```bash
sforge test                    # Run all tests
sforge test -vvvv              # Verbose output
sforge test --match-test testTransfer  # Run specific test
```

### Deploying

```bash
# Deploy to devnet
sforge create src/MyContract.sol:MyContract \
    --rpc-url https://node-2.seismicdev.net/rpc \
    --private-key $PRIVATE_KEY

# Deploy with script
sforge script script/Deploy.s.sol \
    --rpc-url https://node-2.seismicdev.net/rpc \
    --private-key $PRIVATE_KEY \
    --broadcast
```

### Local development

```bash
sanvil  # Starts local Seismic node on http://127.0.0.1:8545
```

## Common Mistakes

1. **Using `public` on shielded variables** — `suint256 public x` won't compile. Use `private` or `internal` and write an explicit getter with access control.
2. **Emitting shielded values directly** — `emit Transfer(from, to, amount)` where `amount` is `suint256` won't compile. Encrypt with the AES-GCM precompile first, then emit the encrypted bytes.
3. **Casting shielded to unshielded carelessly** — `uint256(myShieldedValue)` makes it visible in execution traces. Only cast when you intentionally want to make a value public.
4. **Using `forge` instead of `sforge`** — Standard Foundry tools don't understand shielded types. Always use `sforge`, `sanvil`, `ssolc`.
5. **Forgetting signed reads** — Any `view` function that checks `msg.sender` requires clients to use a signed read. Document this in your function's NatSpec.

## Networks

| Network        | Chain ID | RPC URL                             |
| -------------- | -------- | ----------------------------------- |
| Devnet         | 5124     | `https://node-2.seismicdev.net/rpc` |
| Devnet (WS)    | 5124     | `wss://node-2.seismicdev.net/ws`    |
| Local (sanvil) | 31337    | `http://127.0.0.1:8545`             |

Faucet: https://faucet-2.seismicdev.net/

## Links

- [Shielded Types](https://docs.seismic.systems/seismic-solidity/shielded-types/)
- [Precompiles Reference](https://docs.seismic.systems/reference/precompiles)
- [Migrating from Ethereum](https://docs.seismic.systems/networks/migrating-from-ethereum)
- [GitHub: seismic-foundry](https://github.com/SeismicSystems/seismic-foundry)
````

## What this teaches Claude

- **Shielded type syntax and behavior** — Claude will use `suint256`/`saddress`/`sbool` correctly instead of inventing non-existent types
- **Toolchain commands** — Claude will use `sforge`/`sanvil`/`ssolc` instead of standard Foundry tools
- **Precompile addresses and calling convention** — Claude can write encrypted event patterns without guessing
- **Access control + signed reads** — Claude will mark shielded getters as requiring signed reads
- **Event encryption** — Claude will use the ECDH → HKDF → AES-GCM pipeline instead of emitting raw shielded values

## Customizing

After pasting the template:

- Replace `[Your Project Name]` with your project name
- Add your contract file paths under a `## Project Structure` section if you want Claude to know where things live
- Add any project-specific patterns (e.g., your own base contract, custom access control, specific precompile usage)
