---
icon: cubes
---

# Predeploys

A predeploy is an EVM contract whose runtime bytecode and initial storage are
included directly in a network's genesis allocation. It is available from
block 0 at a fixed address, without a deployment transaction.

{% hint style="info" %}
The exact predeploy set can differ between Seismic networks and protocol
versions. A network's genesis file is the source of truth. The checked-in
[genesis contract manifest](https://github.com/SeismicSystems/seismic-reth/blob/seismic/crates/seismic/chainspec/res/genesis/manifest.toml)
defines the Solidity contracts included by Seismic's genesis builder. Use
`eth_getCode` to confirm that a particular address contains code on the
network you are using.
{% endhint %}

## Seismic genesis predeploys

These Seismic contracts are installed by placing their runtime bytecode at a
fixed address in the genesis allocation. Constructors do not run during this
process, so any required initial state must also be supplied by the genesis
configuration.

| Contract | Address | Purpose |
| --- | --- | --- |
| Deposit contract | `0x00000000219ab540356cBB839Cbe05303d7705Fa` | Accepts validator deposits and maintains the deposit Merkle tree |
| Protocol parameters | `0x0000000000000000000000000000506172616D73` | Stores owner-managed protocol configuration |
| Shielded delegation account | `0x0000000000000000000000000000000000002001` | Experimental EIP-7702 delegation implementation with shielded session keys |
| UpgradeOperator | `0x1000000000000000000000000000000000000001` | Stores TEE measurements permitted to participate in the network |
| MultisigUpgradeOperator | `0x1000000000000000000000000000000000000002` | Authorizes changes to UpgradeOperator |
| Directory | `0x1000000000000000000000000000000000000004` | Stores account viewing keys used by SRC20 encrypted events |
| Intelligence | `0x1000000000000000000000000000000000000005` | Manages intelligence providers and encrypts data to their registered keys |
| Operations sentinel | `0x1000000000000000000000000000000000000006` | Reserved target for node operations authorization messages; not an application contract |

`UpgradeOperator` and `MultisigUpgradeOperator` are intended for TEE
measurement admission. Despite their names, they do not upgrade contract
bytecode. They are deployed at their fixed addresses but are not currently
used by the network.

The AES Solidity library is not an AES precompile. Current contracts call the
AES and HKDF precompiles through internal `CryptoUtils` functions and do not
link to this deployed library. The predeploy remains present for existing
networks but is deprecated and slated for deletion from future genesis
configurations.

The operations sentinel is intercepted by Seismic node software. Its fixed
address and ABI identify an operations request, while its behavior is not
provided by the bytecode at that address.

## Ethereum system predeploys

Seismic also includes Ethereum-standard system contracts needed by the
execution and consensus protocols.

| Contract | Address | Standard |
| --- | --- | --- |
| Beacon block roots | `0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02` | EIP-4788 |
| Withdrawal requests | `0x00000961Ef480Eb55e80D19ad83579A64c007002` | EIP-7002 |

These contracts are part of protocol execution. Their bytecode should change
only as part of a coordinated protocol upgrade, not through an application
administrator.

## Upgradeability

The current genesis configuration installs the listed runtime bytecode
directly at each predeploy address. It does not currently put the Seismic
predeploys behind EIP-1967 proxies.

Upgradeability is a property of an individual predeploy, not of the address
range itself. Stateful Seismic services may move behind stable proxies in a
future genesis version, while consensus-standard contracts, linked libraries,
and EIP-7702 delegation implementations should remain direct code deployments.
