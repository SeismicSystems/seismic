---
description: Optional Claude Code workflow skills for common Seismic tasks
icon: wand-magic-sparkles
---

# Workflow Skills

Claude Code supports [skills](https://docs.anthropic.com/en/docs/claude-code/skills) — reusable task instructions that you can invoke with slash commands. Skills are stored as markdown files in your project and provide step-by-step guidance for specific workflows.

These skills are optional. The [CLAUDE.md templates](templates/) cover the core API knowledge that Claude needs. Skills add structured workflows for tasks you do repeatedly.

## Setup

To add a skill to your project:

1. Create the directory: `.claude/skills/`
2. Save the skill file with the path shown below each skill
3. Invoke it in Claude Code with `/skill-name` or describe the task and Claude will follow the steps

---

## Deploy Shielded Contract

A step-by-step workflow for deploying a Seismic smart contract with `sforge`.

**Save to:** `.claude/skills/deploy-shielded-contract/SKILL.md`

````markdown
# Deploy Shielded Contract

Deploy a Seismic smart contract using sforge.

## Steps

1. **Verify toolchain** — Run `sforge --version` to confirm the Seismic toolchain is installed. If not, run:

   ```bash
   sfoundryup
   source ~/.zshenv  # or ~/.bashrc
   ```

2. **Compile** — Run `sforge build` and fix any compiler errors. Pay attention to:
   - Shielded types (`suint256`, `saddress`, `sbool`) must not be `public`
   - Events cannot emit shielded types directly — encrypt first

3. **Run tests** — Run `sforge test -vvvv` and ensure all tests pass.

4. **Choose network** — Ask which network to deploy to:
   - **Local (sanvil):** `http://127.0.0.1:8545` — start sanvil first with `sanvil`
   - **Devnet:** `https://node-2.seismicdev.net/rpc` — needs funded wallet from https://faucet-2.seismicdev.net/

5. **Deploy** — Run the deployment:

   ```bash
   sforge create src/<ContractFile>.sol:<ContractName> \
       --rpc-url <RPC_URL> \
       --private-key $PRIVATE_KEY
   ```

   Or if using a deployment script:

   ```bash
   sforge script script/Deploy.s.sol \
       --rpc-url <RPC_URL> \
       --private-key $PRIVATE_KEY \
       --broadcast
   ```

6. **Verify** — Confirm the contract address in the output. Check it on the explorer if deploying to devnet: https://explorer-2.seismicdev.net/

7. **Update project** — Add the deployed contract address to the project's configuration or constants file.
````

---

## Debug Shielded Transaction

A diagnostic checklist for when a shielded transaction fails or behaves unexpectedly.

**Save to:** `.claude/skills/debug-shielded-tx/SKILL.md`

```markdown
# Debug Shielded Transaction

Systematic checklist for debugging a failed or unexpected shielded transaction.

## Diagnostic Steps

1. **Check transaction type** — Is the transaction type `0x4A` (Seismic)?
   - If not, the calldata was sent in plaintext. Ensure you're using a shielded wallet client (`createShieldedWalletClient`, `SeismicSignedProvider`, or `w3.seismic.contract().write`).

2. **Check gas parameters** — Seismic uses **legacy gas** (gas_price + gas_limit):
   - Do NOT use `maxFeePerGas` or `maxPriorityFeePerGas`
   - Try increasing the gas limit (e.g., `gas: 500_000n` or `gas: 1_000_000n`)

3. **Check encryption** — The TEE public key must be fetched before sending:
   - Client creation is async — verify you `await`ed the client constructor
   - Check RPC connectivity: `curl -s <RPC_URL> -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"seismic_getTeePublicKey","params":[],"id":1}'`
   - The response should contain a 33-byte (66 hex chars) compressed public key

4. **Check signed reads** — If a `view` function that checks `msg.sender` reverts:
   - Standard `eth_call` zeroes the sender on Seismic
   - Use signed read: `contract.read.*` (seismic-viem), `provider.seismic_call()` (seismic-alloy), or `.read` namespace (seismic-python)

5. **Check nonce** — Seismic transactions use separate nonce tracking:
   - If getting nonce errors, try resetting nonce or fetching the latest: `eth_getTransactionCount`

6. **Check block window** — Seismic blocks are ~600ms:
   - Transactions may be included faster than expected
   - Receipt polling should use short intervals

7. **Check contract state** — If the tx succeeds but reads return unexpected values:
   - Shielded storage is not readable via `eth_getStorageAt`
   - Use a signed read through the contract's getter function
   - Verify the getter has proper access control (`require(msg.sender == ...)`)

8. **Check the explorer** — For devnet transactions: https://explorer-2.seismicdev.net/
   - Shielded data will appear as zeros in traces (this is expected)

## Common Error Patterns

| Symptom                      | Likely cause                                  |
| ---------------------------- | --------------------------------------------- |
| "execution reverted" on read | Using plain `eth_call` instead of signed read |
| Calldata visible on-chain    | Using standard tx type instead of 0x4A        |
| "insufficient funds"         | Wallet not funded — use faucet                |
| Nonce too low/high           | Pending transactions or nonce mismatch        |
| Empty return on getter       | Getter needs `msg.sender` check + signed read |
```

---

## Migrate ERC20 to SRC20

A guided workflow for converting a standard ERC20 token to a Seismic SRC20 (shielded) token.

**Save to:** `.claude/skills/migrate-erc20-src20/SKILL.md`

````markdown
# Migrate ERC20 to SRC20

Convert a standard ERC20 token contract to a shielded SRC20 token on Seismic.

## Steps

### 1. Identify what to shield

Typical candidates:

- `balanceOf` mapping → `mapping(address => suint256) private balanceOf`
- `allowance` mapping → `mapping(address => mapping(address => suint256)) private allowance`
- Transfer amounts in function parameters

Keep public:

- `totalSupply`, `name`, `symbol`, `decimals` — these are non-sensitive metadata

### 2. Update types

Replace shielded state variables:

```solidity
// Before (ERC20)
mapping(address => uint256) public balanceOf;

// After (SRC20)
mapping(address => suint256) private balanceOf;
```

### 3. Remove public getters, add access-controlled getters

```solidity
// Replace auto-generated public getter with:
function getBalance(address account) external view returns (suint256) {
    require(msg.sender == account, "Only owner can view balance");
    return balanceOf[account];
}
```

Document that callers must use **signed reads** for these functions.

### 4. Update events

Shielded types cannot be emitted directly. Encrypt sensitive event data:

```solidity
event EncryptedTransfer(
    address indexed from,
    address indexed to,
    bytes encryptedAmount
);
```

Use the ECDH → HKDF → AES-GCM precompile pattern to encrypt the amount for the recipient. See the Seismic docs on [Encrypted Events](https://docs.seismic.systems/seismic-solidity/events) for the full pattern.

### 5. Update transfer/approve logic

- Arithmetic on `suint256` works the same as `uint256`
- Comparisons work the same (`>=`, `==`, etc.)
- The compiler routes to `CLOAD`/`CSTORE` automatically

### 6. Update tests

```bash
sforge test -vvvv
```

Key things to test:

- Shielded balance updates correctly after transfer
- Access control on getters rejects unauthorized callers
- Encrypted events are emitted (check for event signature, not plaintext values)
- Allowance mechanism works with shielded types

### 7. Update client code

Replace standard contract interaction with shielded equivalents:

- Use `contract.write.transfer(...)` instead of `contract.functions.transfer(...).transact()`
- Use `contract.read.getBalance(...)` instead of `contract.functions.balanceOf(...).call()`
- Listen for `EncryptedTransfer` events and decrypt client-side

### 8. Deploy and verify

Deploy with `sforge create` or `sforge script` to the target network.
````
