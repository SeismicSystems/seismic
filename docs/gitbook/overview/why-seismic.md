---
icon: question
---

# Why Seismic

## One letter changes everything

Here is a standard ERC20 token:

```solidity
// Standard ERC20 — balances visible to everyone
mapping(address => uint256) public balanceOf;

function transfer(address to, uint256 amount) public {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

Here is the Seismic equivalent -- an SRC20 with fully private balances:

```solidity
// Seismic SRC20 — balances shielded by default
mapping(address => suint256) balanceOf;    // uint256 → suint256

function transfer(address to, suint256 amount) public {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

The changes:

* `uint256` becomes `suint256`
* The `public` visibility modifier is removed (shielded types cannot be returned from public getters)

That is it. The business logic is identical. The compiler handles the rest -- routing shielded values to encrypted storage, ensuring they never appear in traces or state reads by external observers. Anyone looking at this contract's storage, transaction calldata, or execution traces sees `0x000` where the actual values should be.

## The transparency problem

Blockchains are public ledgers. Every balance, every transaction, every contract interaction is visible to anyone with a block explorer. This transparency was a design choice -- but it creates real problems:

* **Front-running and MEV extraction.** Bots watch the mempool, see your trade, and sandwich it for profit. On Ethereum, MEV extraction costs users billions annually.
* **Competitive intelligence leaks.** If your protocol holds assets on-chain, competitors can see your treasury, your positions, and your strategy in real time.
* **Privacy violations for end users.** When Alice sends tokens to Bob, everyone can see how much she holds, how much she sent, and build a complete transaction graph linking her activity.
* **Business logic exposure.** Contract state is fully readable. Pricing algorithms, liquidation thresholds, and internal parameters are all public.

Traditional finance operates with confidentiality by default. Blockchain flips that assumption, and it limits what you can build.

## Why existing privacy solutions fall short

Several projects have tried to solve this. None of them let you stay in Solidity.

| Approach                           | Limitation                                                                                                                       |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **ZK chains** (Aztec, Aleo, Mina)  | Require new languages (Noir, Leo, o1js). You leave Solidity, Foundry, and the entire EVM ecosystem behind.                       |
| **Mixers** (Tornado Cash)          | Only work for transfers. You can obscure the sender, but contract state is still public. No programmable privacy.                |
| **L2 privacy layers**              | Sacrifice composability. Contracts on the privacy layer cannot natively interact with contracts on the base chain.               |
| **Homomorphic encryption** (fhEVM) | Computationally expensive. Operations on encrypted data are orders of magnitude slower, limiting what you can practically build. |

Each of these approaches forces a tradeoff: learn a new language, accept limited functionality, or live with performance constraints.

## The Seismic approach

Seismic solves on-chain privacy with two innovations working together:

### Shielded types in the compiler

Seismic extends the Solidity compiler with **shielded types**: `suint`, `sint`, `sbool`, and `saddress`. The `s` prefix marks a value as private. Under the hood, these compile to `CLOAD` and `CSTORE` opcodes (instead of the standard `SLOAD`/`SSTORE`), which route data through encrypted storage.

No new language. No new programming model. Just a one-letter prefix on the types you already know.

### TEE-based execution

Seismic nodes run inside **Trusted Execution Environments** (TEEs) using Intel TDX. The TEE creates a hardware-enforced boundary: even the node operator cannot read the data being processed. Transactions are encrypted before they hit the network and decrypted only inside the TEE for execution.

Together, these two layers provide privacy at the language level and enforcement at the hardware level.

## What does not change

Seismic is designed so that everything around the privacy layer stays familiar:

* **Same language.** Standard Solidity, with shielded types added. No new syntax beyond the `s` prefix.
* **Same tooling.** `sforge`, `sanvil`, and `ssolc` are forks of Foundry's `forge`, `anvil`, and `solc`. Your workflow does not change.
* **Same client libraries.** `seismic-viem` extends Viem. `seismic-react` extends React hooks. `seismic-alloy` extends Alloy for Rust.
* **Same standards.** ERC20 becomes SRC20. The interface is the same. The deployment flow is the same. The difference is that balances are private.
* **Same deployment flow.** Write, test, deploy. The commands are `sforge build`, `sforge test`, `sforge create`. If you have deployed to Ethereum, you can deploy to Seismic.
