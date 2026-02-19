---
description: >-
  \Build a private ERC20 token where balances and transfers are hidden from
  observers
icon: coins
layout:
  width: default
  title:
    visible: true
  description:
    visible: true
  tableOfContents:
    visible: true
  outline:
    visible: true
  pagination:
    visible: true
  metadata:
    visible: true
  tags:
    visible: true
---

# Build a Private SRC-20 Token

In this tutorial, you will build a fully functional private ERC20 token -- an SRC20 -- where balances, transfer amounts, and allowances are all shielded from external observers. Anyone watching the chain sees `0x000` instead of actual values, yet the token behaves exactly like a standard ERC20 from the user's perspective.

## What you'll build

By the end of this tutorial you will have:

* An SRC20 smart contract with shielded balances, transfers, and allowances
* Encrypted transfer events that only the sender and recipient can decrypt
* A signed-read pattern that lets users check their own balance without revealing it
* Compliance-ready access control through Intelligence Contracts
* A React frontend that connects everything end-to-end

## What makes this special

The contract changes from a standard ERC20 to an SRC20 are remarkably small. The core of it is changing `uint256` to `suint256` for balances, amounts, and allowances. The transfer logic, require checks, and overall structure stay almost identical. Seismic's compiler handles the rest -- routing reads and writes through shielded storage automatically.

This is the power of Seismic's approach: privacy is a type-level annotation, not a protocol-level rewrite.

## Prerequisites

Before starting, make sure you have:

* **Seismic development tools installed** -- `sforge`, `sanvil`, and `ssolc`. See the [Installation guide](../../gitbook/getting-started/installation.md) if you have not set these up yet.
* **Solidity familiarity** -- You should be comfortable writing and reading Solidity contracts.
* **Basic understanding of ERC20** -- You should know what `balanceOf`, `transfer`, `approve`, and `transferFrom` do.

## What you'll learn

| Chapter | Topic                                                                 | Key concept                                   |
| ------- | --------------------------------------------------------------------- | --------------------------------------------- |
| 1       | [ERC20 to SRC20](erc20-to-src20.md)                                   | Shielded types and the minimal diff           |
| 2       | [Shielded Balances and Transfers](shielded-balances-and-transfers.md) | `suint256` in practice, testing with `sforge` |
| 3       | [Encrypted Events](encrypted-events.md)                               | AES-GCM precompiles for private event data    |
| 4       | [Signed Reads](signed-reads-for-balance-checking.md)                  | Letting users view their own balance securely |
| 5       | [Intelligence Contracts](intelligence-contracts.md)                   | Compliance-compatible access control          |
| 6       | [Building the Frontend](building-the-frontend.md)                     | React integration with `seismic-react`        |

## Tutorial structure

**Chapter 1** starts with a side-by-side comparison of a standard ERC20 and the SRC20 version, walking through every changed line. **Chapter 2** dives into the implementation of shielded transfers, allowances, and minting, including how to test with `sforge`. **Chapter 3** tackles encrypted events -- since shielded types cannot appear in event parameters, you will use AES-GCM precompiles to encrypt sensitive data before emitting. **Chapter 4** introduces signed reads, the mechanism that lets users query their own balance without exposing it. **Chapter 5** adds compliance through Intelligence Contracts, showing how authorized roles can inspect shielded state. **Chapter 6** brings it all together with a React frontend using `seismic-react`.

Each chapter builds on the previous one. By the end, you will have a complete, deployable private token with a working frontend.
