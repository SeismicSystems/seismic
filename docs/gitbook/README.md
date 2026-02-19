---
description: Private by Default. Familiar by Design.
icon: hand-wave
cover:
  light: ../.gitbook/assets/light.png
  dark: ../.gitbook/assets/dark.png
coverY: 0
---

# Welcome to Seismic!

**Seismic is an EVM blockchain with native on-chain privacy.** Write Solidity. Use Foundry. Deploy with Viem. The only difference: your users' data stays private.

***

### One letter changes everything

The difference between a public ERC20 and a private SRC20 is one letter:

```solidity
// Standard ERC20 — balances visible to everyone
mapping(address => uint256) public balanceOf;

function transfer(address to, uint256 amount) public {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

```solidity
// Seismic SRC20 — balances shielded by default
mapping(address => suint256) balanceOf;    // uint256 → suint256

function transfer(address to, suint256 amount) public {  // uint256 → suint256
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

That `s` prefix tells the Seismic compiler to encrypt the value at rest, in transit, and during execution. Observers see `0x000` instead of actual balances and amounts. Everything else — the Solidity syntax, the EVM execution model, the deployment flow — stays exactly the same.

***

### What you can build

* **Private tokens** — ERC20s where balances and transfer amounts are hidden from observers
* **Confidential DeFi** — AMMs and lending protocols where positions, prices, and liquidation thresholds stay private
* **Compliant finance** — Privacy with built-in access control so regulators can verify without exposing user data
* **Private voting** — On-chain governance where votes are secret until tallied

***

### 3-minute quickstart

Already have Rust and the [Seismic tools installed](../getting-started/installation.md)?

```bash
git clone "https://github.com/SeismicSystems/seismic-starter.git"
cd seismic-starter/packages/contracts
sforge test -vv
```

You just ran shielded contract tests locally. See the [full quickstart](../getting-started/quickstart.md) for next steps.

***

### Find what you need

| I want to...                         | Go to                                                                    |
| ------------------------------------ | ------------------------------------------------------------------------ |
| Understand why Seismic exists        | [Why Seismic](overview/why-seismic.md)                                   |
| See how it works under the hood      | [How Seismic Works](overview/how-seismic-works.md)                       |
| Set up my dev environment            | [Installation](../getting-started/installation.md)                       |
| Run my first shielded contract       | [Quickstart](../getting-started/quickstart.md)                           |
| Build a complete app step by step    | [Walnut App Tutorial](/broken/pages/BN5OqFBvn79QEqM2Y5JX)                |
| Build a private ERC20 token          | [SRC20 Tutorial](tutorials/src20/)                                       |
| Learn about shielded types           | [Shielded Types](../seismic-solidity/seismic-solidity/shielded-types.md) |
| Integrate a frontend                 | [Client Libraries](client-libraries/overview.md)                         |
| Deploy to testnet or mainnet         | [Networks](../reference/networks/testnet.md)                             |
| Understand the transaction lifecycle | [Seismic Transaction](reference/seismic-transaction.md)                  |
| Run a node                           | [Node Operator FAQ](reference/node-operator-faq.md)                      |

***

### Pre-requisite knowledge

Our documentation assumes some familiarity with blockchain app development. Before getting started, it'll help if you're comfortable with:

* [Solidity](https://www.soliditylang.org/)
* [Foundry](https://getfoundry.sh/)
* [Viem](https://viem.sh/)

If you're new to blockchain app development or need a refresher, we recommend starting out with the [CryptoZombies](https://cryptozombies.io/en/course) tutorial.

***

### Work with us

If you might benefit from direct support from the team, please don't hesitate to reach out to `dev@seismic.systems`. We pride ourselves in fast response time.

You can also check out our [X account](https://x.com/SeismicSys) for the latest updates, or join our [Discord](https://discord.gg/XSPNseXCvW) community.
