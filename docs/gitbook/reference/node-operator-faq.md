---
icon: compass-drafting
---

# Node Operator FAQ

### General

#### Seismic nodes have to run inside a TEE. Why? and what does that mean for node operators?

Seismic nodes run inside TEEs so we can verify that they are running the correct software via remote attestation. If someone were to deploy a node that allowed them to view network secrets, it would be rejected by other nodes, and therefore never receive any sensitive data.&#x20;

As a result, all node operators have to be running the exact same versions of the code, including reth parameters. If you are an RPC provider partnering with us, and need nodes to run with specific settings, please contact our team – we'll see how we can help. While we have nothing in place to support this now, we can prioritize features to make it easier for you to run your business

#### Is Seismic a ZK Chain?

No. Seismic uses trusted execution environments (TEE) via Intel TDX for privacy, not zero-knowledge proofs.

#### Does Seismic support light nodes, full nodes or archival nodes?

Seismic currently supports archival nodes only

#### How fast does storage grow?

* **Current size**: TBD (network has not yet launched)
* **Archive node**: 1TB+ storage recommended initially
* **Growth rate**: Will depend on network activity; approximately 12 hours of sync time expected for first year of operation

Detailed storage projections will be published after mainnet launch

#### How do I run a node?

There are instructions to deploy a node in our [deploy](https://github.com/SeismicSystems/deploy) repo. There are two steps:

1. Build (optional): you can build the image yourself using our Python scripts in the deploy repo. Alternatively we will be hosting images that we've built, along with the measurements generated. When we do this, you can download the image from the releases page of that repo. The basic command is: `python3 -m yocto.cli --build --logs`
2. Deploy: once you have an image, you can deploy it to Azure using our Python tooling. The basic command is: `python3 -m yocto.genesis_deploy -a 20251017221200 -n 1`&#x20;

Soon we will publish more detailed documentation on our Python tooling, which will allow you to customize the deploy

### Hardware requirements

#### Cloud hosting

Seismic uses Azure's Confidential Computing with Intel TDX to run our nodes. We are also planning to support bare metal TDX as well

#### Recommended specs

* CPU: 4+ vCPUs
* Memory: 16+GB RAM
* Storage: 1TB
* Azure Confidential virtual machines (TDX) with secure boot & TPM enabled
  * Example instance: EC4es v5
* Security: Confidential VM with secure boot and vTPM (NonPersistedTPM)
* SKU: `standard_lrs` with `ConfidentialVM_NonPersistedTPM` security type

### RPC

#### Are there rate limits on RPC calls?

No rate limits are currently imposed by the protocol itself, though node operators may implement their own.

#### Is there an RPC parameter to set the maximum fee cap?

Yes, `--rpc.txfeecap`. We use reth's default, which is 1.0 units of the native token (e.g. 1.0 ETH on testnet)

#### Is there a maximum payload size for RPC requests?&#x20;

Yes, this is controlled through the arg `--rpc.max-response-size`. We use reth's default, which is 160MB

#### Is there a limit on the batch count for RPC requests?

No. Just like in reth, there's no limit on batch count. The only limit comes from total payload size (above)

#### What is the maximum size for `eth_getLogs` responses?

This is the same as reth's maximum payload size for general RPC requests: 160MB

#### Does Seismic support log look back?

Yes, archival nodes support complete log look back and retrieval of contract events from the beginning of the chain

#### What sync mode should I use for fetching logs?

We only support archival nodes. Make RPC calls to them with block filters

#### What are the heaviest RPC methods?

The most resource-intensive RPC methods are:

* **`eth_getLogs`** with large block ranges or many matching events
* **Tracing calls** (e.g., `debug_traceTransaction`, `trace_*` methods) with complex geth tracers

#### Are block height indicators available?

Yes, use `eth_blockNumber` to check current block height and sync progress

#### Are there recommended caching rules for RPC methods?

We haven't thought about this yet

### Operations

#### How often are hard forks expected?

No hard forks have occurred yet (network is pre-mainnet). The frequency of future hard forks is TBD, but all upgrades will be communicated via Twitter, Discord, and direct partner outreach

All changes will be deployed to testnet before mainnet

#### Does the node handle SIGTERM gracefully?

Individual processes do support this. However because the node has to run inside a TEE, the correct way to restart a node is to reboot the machine. Relevant processes will automatically spawn on boot

**Expected restart time**: About 1 minute from machine reboot

#### Have there been any major outages?

Mainnet has not launched yet, so no. In testnet, various incidents have occurred, but these have been resolved prior to public release
