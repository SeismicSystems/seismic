---
description: A list of all our codebases
icon: code
---

# Codebases

## [Summit](https://github.com/SeismicSystems/summit)

Our consensus client. Built on top of [Commonware's](https://commonware.xyz/) [primitives](https://github.com/commonwarexyz/monorepo)

## [Seismic Solidity](https://github.com/SeismicSystems/seismic-solidity)

Our fork of [Solidity](https://soliditylang.org/). We added a set of types & opcodes for shielded computation

## [Enclave](https://github.com/SeismicSystems/enclave)

Codebase for encryption, TEE & on-chain verification

## Execution layer

Most of the repositories here are forks of the reth stack

### **Alloy**

#### [Seismic Alloy Core](https://github.com/SeismicSystems/seismic-alloy-core)

fork of [alloy-rs/core](https://github.com/alloy-rs/core)

* This is the repo that depends on nothing else
* Upstream: version 1.1.2, commit [`e55993f`](https://github.com/alloy-rs/core/tree/e55993f69d91f36fdb501d54e11a8265f90e42c1)&#x20;

#### [Seismic Alloy](https://github.com/SeismicSystems/seismic-alloy)

Analogous to [alloy-rs/op-alloy](https://github.com/alloy-rs/op-alloy), but not a fork of it

* Depends on:
  * seismic-alloy-core
  * alloy-rs/alloy

#### [Seismic Trie](https://github.com/SeismicSystems/seismic-trie)

fork of [alloy-rs/trie](https://github.com/alloy-rs/trie)

* Depends on seismic-alloy-core
* Upstream: version 0.8.1, commit [`a098d3f`](https://github.com/alloy-rs/trie/commit/a098d3f6b212dc8e44a2d4886be9c0f6b2ec63e9)

### **REVM**

#### [Seismic REVM](https://github.com/SeismicSystem/seismic-revm)

fork of [bluealloy/revm](https://github.com/bluealloy/revm)

* Depends on:
  * seismic-alloy-core
  * seismic-enclave
* Upstream: version 23.1.0, commit [`b287ce02`](https://github.com/bluealloy/revm/commit/b287ce025565c6f9206d5959c08acc401c8be5d4)

#### [Seismic EVM](https://github.com/SeismicSystems/seismic-evm)

fork of [alloy-rs/evm](https://github.com/alloy-rs/evm)

* Depends on:
  * alloy-rs/alloy
  * seismic-alloy
  * seismic-alloy-core
  * seismic-revm
* Upstream: version 0.9.1, commit [`1c4f35c`](https://github.com/alloy-rs/evm/commit/1c4f35ca45a4d32ec6929be0943ff59618fe8088)&#x20;

#### [Seismic REVM Inspectors](https://github.com/SeismicSystems/seismic-revm-inspectors)

fork of [paradigmxyz/revm-inspectors](https://github.com/paradigmxyz/revm-inspectors)

* Depends on:
  * seismic-alloy-core
  * alloy-rs/alloy
  * seismic-revm
* Upstream: version 0.22.3, commit [`dd283db`](https://github.com/paradigmxyz/revm-inspectors/commit/dd283dbeb19fd94a8e6de60ef9acfaad042f4b94)

### [Reth](https://github.com/SeismicSystems/seismic-reth)

fork of [paradigmxyz/reth](https://github.com/paradigmxyz/reth)

* Depends on:
  * seismic-alloy-core
  * seismic-alloy
  * alloy-rs/alloy
  * alloy-trie
  * seismic-revm
  * seismic-evm
  * seismic-revm-inspectors
* Upstream: version 1.2.1, commit [`3c0b3df8`](https://github.com/foundry-rs/foundry/commit/3c0b3df8f8ef8800a10912ce5a9dcd9eb7e971ff)

### **Foundry**

#### [Seismic Compilers](https://github.com/SeismicSystems/seismic-compilers)

fork of [foundry-rs/compilers](https://github.com/foundry-rs/compilers)

* Depends on:
  * seismic-alloy-core
* Upstream: version 0.16.1, commit [`ec745cec`](https://github.com/foundry-rs/compilers/commit/ec745cecb9c19e67140704bcb359928851ca48c5)

#### [Seismic Foundry Fork DB](https://github.com/SeismicSystems/seismic-foundry-fork-db)

fork of [foundry-rs/foundry-fork-db](https://github.com/foundry-rs/foundry-fork-db)

* Depends on:
  * seismic-alloy-core
  * alloy-rs/alloy
  * seismic-revm
  * seismic-alloy (only for seismic-prelude)
* Upstream: version 0.14.0, commit [`0fe2b2a`](https://github.com/foundry-rs/foundry-fork-db/commit/0fe2b2a0bb7059ed44d8401ed3c9d0b9891a87c2)

#### [Seismic Foundry](https://github.com/SeismicSystems/seismic-foundry)

fork of [foundry-rs/foundry](https://github.com/foundry-rs/foundry)

* Depends on:
  * seismic-alloy-core
  * seismic-alloy
  * alloy-rs/alloy
  * alloy-trie
  * seismic-revm
  * seismic-evm
  * seismic-revm-inspectors
  * seismic-foundry-fork-db
* Upstream: version 1.2.1, commit [`b76d4f66`](https://github.com/SeismicSystems/seismic-reth/commit/b76d4f66179243f6108ee9b1eed231cc854ad924)

## [Seismic Client](https://github.com/SeismicSystems/seismic-client)

A library for building web applications on Seismic. This repo provides two packages that compose with the viem/wagmi stack to interact with the Seismic network:

* seismic-viem: composes with [viem](https://viem.sh/)
* seismic-react: composes with [wagmi](https://wagmi.sh/)



## Deployment

### [Deploy](https://github.com/SeismicSystems/deploy)

A repository containing tools to deploy infrastructure

### Yocto build system

Seismic forked Flashbots' stack for reproducible TEE builds. This includes these repos:

#### [Meta Seismic](https://github.com/SeismicSystems/meta-seismic)

A Yocto layer that configures how the image runs Summit, Reth & the enclave server

#### [Yocto Manifests](https://github.com/SeismicSystems/yocto-manifests)

#### [Yocto Scripts](https://github.com/SeismicSystems/yocto-scripts)



