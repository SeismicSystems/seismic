---
icon: meter-bolt
---

# Chapter 1: Defining the constants and utilities

In this chapter, you will learn about defining and constants and utility functions which we will frequently use throughout the project. _Estimated time: \~10 minutes_

First, navigate to the root of the directory/monorepo and run `bun install`  to install all the dependencies.

Now, navigate to make a `lib` folder inside `packages/cli`with the files `constants.ts` and `utils.ts`   and navigate to it:

```
mkdir -p packages/cli/lib
touch packages/cli/lib/constants.ts packages/cli/lib/utils.ts
cd packages/cli/lib
```

`constants.ts` will contain the constants we use throughout the project with `utils.ts` will contain the necessary utility functions.

Add the following to `constants.ts` :

```typescript
import { join } from 'path'

const CONTRACT_NAME = 'Walnut'
const CONTRACT_DIR = join(__dirname, '../../contracts')

export { CONTRACT_NAME, CONTRACT_DIR }
```

This file centralizes key project constants:

• `CONTRACT_NAME`: The Walnut contract name.

• `CONTRACT_DIR`: Path to the contracts directory.

Now, add the following to `utils.ts` :&#x20;

```typescript
import fs from 'fs'
import { type ShieldedWalletClient, getShieldedContract } from 'seismic-viem'
import { Abi, Address } from 'viem'

async function getShieldedContractWithCheck(
  walletClient: ShieldedWalletClient,
  abi: Abi,
  address: Address
) {
  const contract = getShieldedContract({
    abi: abi,
    address: address,
    client: walletClient,
  })

  const code = await walletClient.getCode({
    address: address,
  })
  if (!code) {
    throw new Error('Please deploy contract before running this script.')
  }

  return contract
}

function readContractAddress(broadcastFile: string): `0x${string}` {
  const broadcast = JSON.parse(fs.readFileSync(broadcastFile, 'utf8'))
  if (!broadcast.transactions?.[0]?.contractAddress) {
    throw new Error('Invalid broadcast file format')
  }
  return broadcast.transactions[0].contractAddress
}

function readContractABI(abiFile: string): Abi {
  const abi = JSON.parse(fs.readFileSync(abiFile, 'utf8'))
  if (!abi.abi) {
    throw new Error('Invalid ABI file format')
  }
  return abi.abi
}

export { getShieldedContractWithCheck, readContractAddress, readContractABI }
```

This file contains utility functions to interact with your Walnut contract:

• `getShieldedContractWithCheck`: Ensures the contract is deployed and returns a shielded contract instance.

• `readContractAddress`: Reads the deployed contract’s address from a broadcast file.

• `readContractABI`: Parses the contract’s ABI from an ABI file for use in interactions.
