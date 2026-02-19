# Table of contents

## Overview

* [Welcome](gitbook/)
* [Why Seismic](the-case-for-seismic/why-seismic.md)
* [How Seismic Works](reference/how-seismic-works.md)
* [Differences from Ethereum](the-case-for-seismic/differences-from-ethereum.md)
* [Use Cases](the-case-for-seismic/use-cases.md)

## Getting Started

* [Installation](gitbook/getting-started/installation.md)
* [Development Toolkit](gitbook/getting-started/development-toolkit.md)
* [Quickstart](gitbook/getting-started/quickstart.md)

## Tutorials

* [Walnut App](/broken/pages/BN5OqFBvn79QEqM2Y5JX)
  * [Understanding the Contract](tutorials/understanding-the-walnut-contract/)
  * [Setting Up Your Project](tutorials/understanding-the-walnut-contract/setting-up-your-project/)
    * [Verify devtool installation](tutorials/understanding-the-walnut-contract/setting-up-your-project/verify-devtool-installation.md)
    * [Create project structure](tutorials/understanding-the-walnut-contract/setting-up-your-project/create-project-structure.md)
    * [Initialize contracts](tutorials/understanding-the-walnut-contract/setting-up-your-project/initialize-contracts.md)
    * [Initialize the CLI](tutorials/understanding-the-walnut-contract/setting-up-your-project/initialize-cli.md)
  * [Writing the Contract](tutorials/understanding-the-walnut-contract/writing-the-contract/)
    * [Ch 1: Making the Kernel](tutorials/understanding-the-walnut-contract/writing-the-contract/chapter-1-making-the-kernel.md)
    * [Ch 2: Making the Shell](tutorials/understanding-the-walnut-contract/writing-the-contract/chapter-2-making-the-shell.md)
    * [Ch 3: Rounds and Access Control](tutorials/understanding-the-walnut-contract/writing-the-contract/chapter-3-rounds-and-access-control.md)
    * [Ch 4: Testing](tutorials/understanding-the-walnut-contract/writing-the-contract/chapter-4-testing.md)
    * [Deploying](tutorials/understanding-the-walnut-contract/writing-the-contract/deploying.md)
  * [Building the CLI](tutorials/understanding-the-walnut-contract/building-the-cli/)
    * [Seismic-viem Primer](tutorials/understanding-the-walnut-contract/building-the-cli/seismic-viem-primer.md)
    * [Ch 1: Constants and Utilities](tutorials/understanding-the-walnut-contract/building-the-cli/chapter-1-constants-and-utilities.md)
    * [Ch 2: Core App Logic](tutorials/understanding-the-walnut-contract/building-the-cli/chapter-2-core-app-logic.md)
    * [Ch 3: Bringing It All Together](tutorials/understanding-the-walnut-contract/building-the-cli/chapter-3-bringing-it-all-together.md)
* [SRC20: Private Token](tutorials/src20/)
  * [ERC20 to SRC20: What Changes](tutorials/src20/erc20-to-src20.md)
  * [Shielded Balances and Transfers](tutorials/src20/shielded-balances-and-transfers.md)
  * [Encrypted Events](tutorials/src20/encrypted-events.md)
  * [Signed Reads](tutorials/src20/signed-reads-for-balance-checking.md)
  * [Intelligence Contracts](tutorials/src20/intelligence-contracts.md)
  * [Building the Frontend](tutorials/src20/building-the-frontend.md)

## Seismic Solidity

* [Shielded Types](gitbook/seismic-solidity/shielded-types/)
  * [suint / sint](gitbook/seismic-solidity/shielded-types/suint-sint.md)
  * [saddress](gitbook/seismic-solidity/shielded-types/saddress.md)
  * [sbool](gitbook/seismic-solidity/shielded-types/sbool.md)
* [Collections](gitbook/seismic-solidity/collections.md)
* [Events](gitbook/seismic-solidity/events.md)
* [Storage](gitbook/seismic-solidity/storage.md)
* [Casting](gitbook/seismic-solidity/casting.md)
* [Best Practices & Gotchas](gitbook/seismic-solidity/best-practices-and-gotchas.md)

## Client Libraries

* [Overview](gitbook/client-libraries/overview.md)
* [Seismic Viem](gitbook/client-libraries/seismic-viem/)
  * [Installation](gitbook/client-libraries/seismic-viem/installation.md)
  * [Shielded Public Client](gitbook/client-libraries/seismic-viem/shielded-public-client.md)
  * [Shielded Wallet Client](gitbook/client-libraries/seismic-viem/shielded-wallet-client.md)
  * [Encryption](gitbook/client-libraries/seismic-viem/encryption.md)
  * [Contract Instance](gitbook/client-libraries/seismic-viem/contract-instance.md)
  * [Signed Reads](gitbook/client-libraries/seismic-viem/signed-reads.md)
  * [Shielded Writes](gitbook/client-libraries/seismic-viem/shielded-writes.md)
  * [Chains](gitbook/client-libraries/seismic-viem/chains.md)
  * [Precompiles](gitbook/client-libraries/seismic-viem/precompiles.md)
* [Seismic React](gitbook/client-libraries/seismic-react/)
  * [Installation](gitbook/client-libraries/seismic-react/installation.md)
  * [Shielded Wallet Provider](gitbook/client-libraries/seismic-react/shielded-wallet-provider.md)
  * [Hooks](gitbook/client-libraries/seismic-react/hooks/)
    * [useShieldedWallet](gitbook/client-libraries/seismic-react/hooks/use-shielded-wallet.md)
    * [useShieldedContract](gitbook/client-libraries/seismic-react/hooks/use-shielded-contract.md)
    * [useShieldedRead](gitbook/client-libraries/seismic-react/hooks/use-shielded-read.md)
    * [useShieldedWrite](gitbook/client-libraries/seismic-react/hooks/use-shielded-write.md)
  * [Wallet Guides](gitbook/client-libraries/seismic-react/wallet-guides/)
    * [RainbowKit](gitbook/client-libraries/seismic-react/wallet-guides/rainbowkit.md)
    * [Privy](gitbook/client-libraries/seismic-react/wallet-guides/privy.md)
    * [AppKit](gitbook/client-libraries/seismic-react/wallet-guides/appkit.md)
* [Seismic Alloy (Rust)](gitbook/client-libraries/seismic-alloy.md)
* [Seismic Python](gitbook/client-libraries/seismic-python.md)

## Networks & Deployment

* [Devnet](tutorials/devnet.md)
* [Testnet](gitbook/networks/testnet.md)
* [Mainnet](gitbook/networks/mainnet.md)
* [Deploy Tools](gitbook/networks/deploy-tools.md)
* [Migrating from Ethereum](reference/migrating-from-ethereum.md)

## Reference

* [The Seismic Transaction](the-case-for-seismic/seismic-transaction.md)
* [Opcodes](reference/opcodes.md)
* [Precompiles](the-case-for-seismic/precompiles.md)
* [RPC Methods](reference/rpc-methods/)
  * [seismic\_getTeePublicKey](reference/rpc-methods/seismic-get-tee-public-key.md)
  * [eth\_call](reference/rpc-methods/eth-call.md)
  * [eth\_sendRawTransaction](reference/rpc-methods/eth-send-raw-transaction.md)
  * [eth\_getStorageAt](reference/rpc-methods/eth-get-storage-at.md)
  * [net\_version](gitbook/reference/rpc-methods/net-version.md)
* [Architecture](reference/architecture.md)
* [Node Operator FAQ](reference/node-operator-faq.md)
* [Codebases](reference/codebases.md)
* [Links & Contact](gitbook/reference/links-and-contact.md)
* [Terms of Service](/broken/pages/ySPArPQXBZFTkxlOXIkq)
* [Privacy Policy](/broken/pages/AOs1gGVRzlkeEBIUkFHQ)
