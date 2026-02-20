---
icon: file-lines
---

# Quick primer: seismic-viem

Before proceeding to write the CLI, you need to be acquainted with some of the functions and utilities used to enable Seismic primitives (e.g. shielded reads, shielded writes etc.) through our client library, `seismic-viem` , which we will be using heavily to write the CLI. The detailed docs for `seismic-viem` can be found [here](https://seismic-docs.netlify.app/). _Estimated time: \~15 minutes_

### Shielded wallet client

The **shielded wallet client** is the shielded/Seismic counterpart of the **wallet client** in `viem` . It is used to enable extended functionality for interacting with shielded blockchain features, wallet operations, and encryption. It can be initialized using the `createShieldedWalletClient` function as follows:

```typescript
const walletClient = await createShieldedWalletClient({
  chain: seismicChain,
  transport: httpTransport,
  privateKey: '0xabcdef...',
})
```

`createShieldedWalletClient` takes in the following parameters:

1. `chain` : a well-defined [Chain](https://github.com/wevm/viem/blob/main/src/types/chain.ts) object
2. `transport` : the method of transport of interacting with the chain (`http` /`ws` along with the corresponding RPC URL)
3. `privateKey`: the private key to create the client for

Once initialized, it can then be used to perform wallet operations or shielded-specific [actions.](https://seismic-docs.netlify.app/seismic-viem/functions/createShieldedWalletClient)

### Shielded contract

A shielded contract instance provides an interface to interact with a shielded contract onchain. It has extended functionality for performing shielded write operations, signed reads, and contract interaction for a **specific contract** performed by a **specific wallet client** that it is initialized with. It can be initialized with the `getShieldedContract` as follows:

```solidity
const contract = getShieldedContract({
  abi: myContractAbi,
  address: '0x1234...',
  client: shieldedWalletClient,
})
```

It takes in the following parameters:

1. `abi` : the ABI of the contract it is interacting with.
2. `address` : the address of the deployed contract it is interacting with.
3. `client` : the shielded wallet client that the interactions are to be performed by.

This function extends the base `getContract` functionality by adding:

* **Shielded write actions** for `nonpayable` and `payable` functions.
* **Signed read actions** for `pure` and `view` functions.
* Proxy-based access to dynamically invoke contract methods.

```solidity
// Perform a shielded write
await contract.write.myFunction([arg1, arg2], { gas: 50000n })
 
// Perform a signed read
const value = await contract.read.getValue()
console.log('Value:', value)
```

We will extensively use **shielded writes (for `shake` ) and shielded reads (for `look()`) in our CLI.**
