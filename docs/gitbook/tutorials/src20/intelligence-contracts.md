---
description: "Add compliance-compatible access control to your private token"
icon: building-columns
---

# Intelligence Contracts

Privacy and compliance are often framed as opposites. Intelligence Contracts show that they are not. This chapter adds role-based access control to the SRC20 so that authorized parties -- auditors, compliance officers, regulators -- can inspect shielded state when required, without compromising privacy for everyone else. _Estimated time: ~15 minutes._

## The concept

An **Intelligence Contract** is a smart contract that can selectively reveal shielded state to authorized parties. The contract stores data privately by default, but includes gated functions that cast shielded values to their unshielded counterparts -- only for callers who hold the right role.

The key insight: the data stays shielded in storage at all times. No plaintext balances are ever written to public state. Authorized parties read the data through signed reads, which means the response is encrypted to their key. The balance is revealed only to the specific authorized caller, not to the world.

## Why this matters

Real-world token issuers need to answer questions like:

- Can a compliance officer verify that an account's balance is below a threshold?
- Can an auditor check aggregate balances across a set of accounts?
- Can the token issuer freeze a specific account if required by law?

Without Intelligence Contracts, privacy is all-or-nothing: either everyone can see everything, or no one can. With Intelligence Contracts, you get selective disclosure -- the right people see the right data, and no one else does.

## Implementation with AccessControl

We will use OpenZeppelin's `AccessControl` to manage roles. This is a battle-tested pattern used across thousands of Ethereum contracts.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/access/AccessControl.sol";

contract SRC20 is AccessControl {
    string public name;
    string public symbol;
    uint8 public decimals = 18;
    uint256 public totalSupply;

    mapping(address => suint256) balanceOf;
    mapping(address => mapping(address => suint256)) allowance;
    mapping(address => bool) public frozen;

    bytes32 public constant COMPLIANCE_ROLE = keccak256("COMPLIANCE_ROLE");
    bytes32 public constant AUDITOR_ROLE = keccak256("AUDITOR_ROLE");

    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);
    event AccountFrozen(address indexed account);
    event AccountUnfrozen(address indexed account);

    constructor(string memory _name, string memory _symbol, uint256 _initialSupply) {
        name = _name;
        symbol = _symbol;
        totalSupply = _initialSupply;
        balanceOf[msg.sender] = suint256(_initialSupply);

        // Deployer gets admin role and can grant other roles
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // --- Standard token functions (with freeze check) ---

    function transfer(address to, suint256 amount) public returns (bool) {
        require(!frozen[msg.sender], "Account frozen");
        require(!frozen[to], "Recipient frozen");
        require(balanceOf[msg.sender] >= amount, "Insufficient balance");

        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        emit Transfer(msg.sender, to, uint256(amount));
        return true;
    }

    function approve(address spender, suint256 amount) public returns (bool) {
        require(!frozen[msg.sender], "Account frozen");
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, uint256(amount));
        return true;
    }

    function transferFrom(address from, address to, suint256 amount) public returns (bool) {
        require(!frozen[from], "Account frozen");
        require(!frozen[to], "Recipient frozen");
        require(allowance[from][msg.sender] >= amount, "Insufficient allowance");
        require(balanceOf[from] >= amount, "Insufficient balance");

        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        emit Transfer(from, to, uint256(amount));
        return true;
    }

    // --- User balance query (signed read) ---

    function getBalance(address account) external view returns (uint256) {
        require(msg.sender == account, "Only owner can view balance");
        return uint256(balanceOf[account]);
    }

    // --- Compliance functions ---

    function complianceBalanceOf(address account) external view returns (uint256) {
        require(
            hasRole(COMPLIANCE_ROLE, msg.sender),
            "Not authorized: requires COMPLIANCE_ROLE"
        );
        return uint256(balanceOf[account]);
    }

    function complianceFreeze(address account) external {
        require(
            hasRole(COMPLIANCE_ROLE, msg.sender),
            "Not authorized: requires COMPLIANCE_ROLE"
        );
        frozen[account] = true;
        emit AccountFrozen(account);
    }

    function complianceUnfreeze(address account) external {
        require(
            hasRole(COMPLIANCE_ROLE, msg.sender),
            "Not authorized: requires COMPLIANCE_ROLE"
        );
        frozen[account] = false;
        emit AccountUnfrozen(account);
    }

    // --- Auditor functions ---

    function auditBalanceOf(address account) external view returns (uint256) {
        require(
            hasRole(AUDITOR_ROLE, msg.sender),
            "Not authorized: requires AUDITOR_ROLE"
        );
        return uint256(balanceOf[account]);
    }
}
```

## Access tiers

The contract implements three levels of access:

| Role                                       | Can do                                     | How they access                                                                             |
| ------------------------------------------ | ------------------------------------------ | ------------------------------------------------------------------------------------------- |
| **Regular user**                           | View their own balance                     | `getBalance(myAddress)` via signed read                                                     |
| **Auditor** (`AUDITOR_ROLE`)               | View any account's balance (read-only)     | `auditBalanceOf(account)` via signed read                                                   |
| **Compliance officer** (`COMPLIANCE_ROLE`) | View any balance, freeze/unfreeze accounts | `complianceBalanceOf(account)` via signed read; `complianceFreeze(account)` via transaction |

### Granting roles

The deployer holds `DEFAULT_ADMIN_ROLE` and can grant roles to other addresses:

```solidity
// Grant compliance role to a specific address
token.grantRole(COMPLIANCE_ROLE, complianceOfficerAddress);

// Grant auditor role
token.grantRole(AUDITOR_ROLE, auditorAddress);
```

In TypeScript:

```typescript
const token = getShieldedContract({
  abi: src20Abi,
  address: SRC20_ADDRESS,
  client: adminWalletClient,
});

// Grant compliance role
await token.write.grantRole([COMPLIANCE_ROLE, complianceOfficerAddress], {
  gas: 100000n,
});
```

### Compliance officer reading a balance

The compliance officer uses a signed read, just like a regular user. The only difference is the function they call:

```typescript
const complianceClient = await createShieldedWalletClient({
  chain: seismicDevnet,
  transport: http(RPC_URL),
  account: privateKeyToAccount(COMPLIANCE_OFFICER_KEY),
});

const complianceToken = getShieldedContract({
  abi: src20Abi,
  address: SRC20_ADDRESS,
  client: complianceClient,
});

// This is a signed read -- response encrypted to the compliance officer's key
const balance = await complianceToken.read.complianceBalanceOf([targetAccount]);
console.log("Account balance:", balance);
```

The balance is returned encrypted to the compliance officer's key. No one else -- not even other compliance officers -- can see this specific response.

## The privacy guarantee

Even with compliance roles in place, the privacy model is strong:

1. **Data stays shielded in storage.** The `balanceOf` mapping always stores `suint256`. No plaintext balances are ever written to public state, regardless of who has what role.

2. **Reads go through signed reads.** Whether it is a user checking their own balance or a compliance officer auditing an account, the query is a signed read. The response is encrypted to the caller's key.

3. **No broadcast disclosure.** When a compliance officer reads a balance, only they learn the value. It is not published on-chain or visible to other observers.

4. **Roles are on-chain and auditable.** The `AccessControl` roles are standard Solidity state. Anyone can verify who holds what role by reading the contract. The role assignments themselves are transparent -- only the shielded data they gate is private.

5. **Freeze is public.** The `frozen` mapping uses `bool`, not `sbool`. This is a deliberate design choice: if an account is frozen, that fact should be publicly verifiable so that counterparties know not to send tokens to it.

{% hint style="info" %}
The role structure shown here is a starting point. In production, you might add time-limited roles, multi-sig requirements for granting compliance access, or on-chain audit logs that record when a compliance officer accessed a balance (without revealing the balance itself).
{% endhint %}
