# Table of contents

## Overview

- [Welcome](README.md)
- [Why Seismic](overview/why-seismic.md)
- [How Seismic Works](overview/how-seismic-works.md)
- [Differences from Ethereum](overview/differences-from-ethereum.md)
- [Use Cases](overview/use-cases.md)

## Getting Started

- [Installation](getting-started/installation.md)
- [Development Toolkit](getting-started/development-toolkit.md)
- [Quickstart](getting-started/quickstart.md)

## Tutorials

- [Walnut App](tutorials/walnut/README.md)
  - [Understanding the Contract](tutorials/walnut/understanding-the-walnut-contract.md)
  - [Setting Up Your Project](tutorials/walnut/setting-up-your-project/README.md)
    - [Verify devtool installation](tutorials/walnut/setting-up-your-project/verify-devtool-installation.md)
    - [Create project structure](tutorials/walnut/setting-up-your-project/create-project-structure.md)
    - [Initialize contracts](tutorials/walnut/setting-up-your-project/initialize-contracts.md)
    - [Initialize the CLI](tutorials/walnut/setting-up-your-project/initialize-cli.md)
  - [Writing the Contract](tutorials/walnut/writing-the-contract/README.md)
    - [Ch 1: Making the Kernel](tutorials/walnut/writing-the-contract/chapter-1-making-the-kernel.md)
    - [Ch 2: Making the Shell](tutorials/walnut/writing-the-contract/chapter-2-making-the-shell.md)
    - [Ch 3: Rounds and Access Control](tutorials/walnut/writing-the-contract/chapter-3-rounds-and-access-control.md)
    - [Ch 4: Testing](tutorials/walnut/writing-the-contract/chapter-4-testing.md)
    - [Deploying](tutorials/walnut/writing-the-contract/deploying.md)
  - [Building the CLI](tutorials/walnut/building-the-cli/README.md)
    - [Seismic-viem Primer](tutorials/walnut/building-the-cli/seismic-viem-primer.md)
    - [Ch 1: Constants and Utilities](tutorials/walnut/building-the-cli/chapter-1-constants-and-utilities.md)
    - [Ch 2: Core App Logic](tutorials/walnut/building-the-cli/chapter-2-core-app-logic.md)
    - [Ch 3: Bringing It All Together](tutorials/walnut/building-the-cli/chapter-3-bringing-it-all-together.md)
- [SRC20: Private Token](tutorials/src20/README.md)
  - [ERC20 to SRC20: What Changes](tutorials/src20/erc20-to-src20.md)
  - [Shielded Balances and Transfers](tutorials/src20/shielded-balances-and-transfers.md)
  - [Encrypted Events](tutorials/src20/encrypted-events.md)
  - [Signed Reads](tutorials/src20/signed-reads-for-balance-checking.md)
  - [Intelligence Contracts](tutorials/src20/intelligence-contracts.md)
  - [Building the Frontend](tutorials/src20/building-the-frontend.md)

## Seismic Solidity

- [Shielded Types](seismic-solidity/shielded-types/README.md)
  - [suint / sint](seismic-solidity/shielded-types/suint-sint.md)
  - [saddress](seismic-solidity/shielded-types/saddress.md)
  - [sbool](seismic-solidity/shielded-types/sbool.md)
- [Collections](seismic-solidity/collections.md)
- [Events](seismic-solidity/events.md)
- [Storage](seismic-solidity/storage.md)
- [Casting](seismic-solidity/casting.md)
- [Best Practices & Gotchas](seismic-solidity/best-practices-and-gotchas.md)

## Client Libraries

- [Overview](client-libraries/overview.md)
- [Seismic Viem](client-libraries/seismic-viem/README.md)
  - [Installation](client-libraries/seismic-viem/installation.md)
  - [Shielded Public Client](client-libraries/seismic-viem/shielded-public-client.md)
  - [Shielded Wallet Client](client-libraries/seismic-viem/shielded-wallet-client.md)
  - [Encryption](client-libraries/seismic-viem/encryption.md)
  - [Contract Instance](client-libraries/seismic-viem/contract-instance.md)
  - [Signed Reads](client-libraries/seismic-viem/signed-reads.md)
  - [Shielded Writes](client-libraries/seismic-viem/shielded-writes.md)
  - [Chains](client-libraries/seismic-viem/chains.md)
  - [Precompiles](client-libraries/seismic-viem/precompiles.md)
- [Seismic React](client-libraries/seismic-react/README.md)
  - [Installation](client-libraries/seismic-react/installation.md)
  - [Shielded Wallet Provider](client-libraries/seismic-react/shielded-wallet-provider.md)
  - [Hooks](client-libraries/seismic-react/hooks/README.md)
    - [useShieldedWallet](client-libraries/seismic-react/hooks/use-shielded-wallet.md)
    - [useShieldedContract](client-libraries/seismic-react/hooks/use-shielded-contract.md)
    - [useShieldedRead](client-libraries/seismic-react/hooks/use-shielded-read.md)
    - [useShieldedWrite](client-libraries/seismic-react/hooks/use-shielded-write.md)
  - [Wallet Guides](client-libraries/seismic-react/wallet-guides/README.md)
    - [RainbowKit](client-libraries/seismic-react/wallet-guides/rainbowkit.md)
    - [Privy](client-libraries/seismic-react/wallet-guides/privy.md)
    - [AppKit](client-libraries/seismic-react/wallet-guides/appkit.md)
- [Seismic Alloy (Rust)](client-libraries/seismic-alloy.md)
- [Seismic Python](client-libraries/seismic-python.md)

## Networks & Deployment

- [Devnet](networks/devnet.md)
- [Testnet](networks/testnet.md)
- [Mainnet](networks/mainnet.md)
- [Deploy Tools](networks/deploy-tools.md)
- [Migrating from Ethereum](networks/migrating-from-ethereum.md)

## Reference

- [The Seismic Transaction](reference/seismic-transaction/README.md)
  - [Tx Lifecycle](reference/seismic-transaction/tx-lifecycle.md)
  - [Signed Reads](reference/seismic-transaction/signed-reads.md)
- [Opcodes](reference/opcodes.md)
- [Precompiles](reference/precompiles.md)
- [Architecture](reference/architecture.md)
- [Node Operator FAQ](reference/node-operator-faq.md)
- [Codebases](reference/codebases.md)
- [Links & Contact](reference/links-and-contact.md)
- [Terms of Service](reference/terms-of-service.md)
- [Privacy Policy](reference/privacy-policy.md)
