---
icon: people
---

# Chapter 3: Bringing it all together

Now that we’ve built the core logic for interacting with the Walnut contract, it’s time to tie everything together into a CLI that runs a multiplayer game session with two players - **Alice and Bob.** In this chapter, you’ll set up the environment variables for multiple players and write `index.ts` to simulate gameplay. _Estimated time: \~20 minutes_

### Set Up Environment Variables

Before running the Walnut game, we need to define environment variables that store important configurations such as the RPC URL, chain ID, and player private keys.

Create a `.env` in `packages/cli` :

```bash
touch .env
```

Open `.env` and paste the following:

```properties
CHAIN_ID=31337
RPC_URL=http://127.0.0.1:8545
ALICE_PRIVKEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
BOB_PRIVKEY=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
```

**What’s Happening Here?**

• `CHAIN_ID=31337` : `31337`is the default chain ID for `sanvil`(your local Seismic node).

• `RPC_URL=http://127.0.0.1:8545` : This is the RPC URL for interacting with the local Seismic node.

• `ALICE_PRIVKEY`and `BOB_PRIVKEY` : These are Alice and Bob’s private keys, allowing them to play the game. (These again are standard test keys provided by `sanvil`)

### Write index.ts

Now, we’ll create the main entry point for our game session. This file will simulate gameplay, initializing players and having them interact with the Walnut contract. Open `packages/cli/src/index.ts`and follow these steps:

#### Import Dependencies

Import the required libraries to read environment variables, define network configurations, and interact with the Walnut contract.

```typescript
import dotenv from 'dotenv'
import { join } from 'path'
import { seismicDevnet } from 'seismic-viem'
import { anvil } from 'viem/chains'

import { CONTRACT_DIR, CONTRACT_NAME } from '../lib/constants'
import { readContractABI, readContractAddress } from '../lib/utils'
import { App } from './app'

// Load environment variables from .env file
dotenv.config()
```

#### Define the main() function

This function initializes the contract and player wallets, then runs the game session.

```typescript
async function main() {
  if (!process.env.CHAIN_ID || !process.env.RPC_URL) {
    console.error('Please set your environment variables.')
    process.exit(1)
  }
```

#### Read Contract Details

The contract’s ABI and deployed address are read from files generated during deployment.

```typescript
  const broadcastFile = join(
    CONTRACT_DIR,
    'broadcast',
    `${CONTRACT_NAME}.s.sol`,
    process.env.CHAIN_ID,
    'run-latest.json'
  )
  const abiFile = join(
    CONTRACT_DIR,
    'out',
    `${CONTRACT_NAME}.sol`,
    `${CONTRACT_NAME}.json`
  )
```

#### Select the blockchain network

Determine whether to use the local `sanvil`node (31337) or the Seismic devnet.

```typescript
  const chain =
    process.env.CHAIN_ID === anvil.id.toString() ? anvil : seismicDevnet
```

#### Define players

Assign Alice and Bob as players with private keys stored in `.env` .

```typescript
  const players = [
    { name: 'Alice', privateKey: process.env.ALICE_PRIVKEY! },
    { name: 'Bob', privateKey: process.env.BOB_PRIVKEY! },
  ]
```

#### Initialize the Game App

Create an `App` instance to interact with the Walnut contract.

```typescript
  const app = new App({
    players,
    wallet: {
      chain,
      rpcUrl: process.env.RPC_URL!,
    },
    contract: {
      abi: readContractABI(abiFile),
      address: readContractAddress(broadcastFile),
    },
  })

  await app.init()
```

#### Simulate the game round by round

The following logic executes two rounds of gameplay between Alice and Bob.

**Round 1 - Alice Plays**

```typescript
  console.log('=== Round 1 ===')
  await app.reset('Alice')
  await app.shake('Alice', 2)
  await app.hit('Alice')
  await app.shake('Alice', 4)
  await app.hit('Alice')
  await app.shake('Alice', 1)
  await app.hit('Alice')
  await app.look('Alice')
```

**Round 2 - Bob Plays**

```typescript
  console.log('=== Round 2 ===')
  await app.reset('Bob')
  await app.hit('Bob')
  await app.shake('Bob', 1)
  await app.hit('Bob')
  await app.shake('Bob', 1)
  await app.hit('Bob')

  // Bob looks at the number in round 2
  await app.look('Bob')
```

**Alice Tries to Look in Round 2 (we expect this to fail since she has contributed in round 1 but not round 2)**

```typescript
  // Alice tries to look in round 2, should fail by reverting
  console.log('=== Testing Access Control ===')
  console.log("Attempting Alice's look() in Bob's round (should revert)")
  try {
    await app.look('Alice')
    console.error('❌ Expected look() to revert but it succeeded')
    process.exit(1)
  } catch (error) {
    console.log('✅ Received expected revert')
  }
}

```

### Execute the main() function

This ensures that the script runs when executed.

```typescript
main()
```

The entire `index.ts` file can be found [here](https://github.com/SeismicSystems/seismic-starter/blob/ameya/baby-walnut/packages/cli/src/index.ts)

### Running the CLI

Now, run the CLI from `packages/cli` by running:

```bash
bun dev
```

You should see something like this as the output:

```
=== Round 1 ===
- Player Alice writing shake()
- Player Alice writing hit()
- Player Alice writing shake()
- Player Alice writing hit()
- Player Alice writing shake()
- Player Alice writing hit()
- Player Alice reading look()
- Player Alice sees number: 7n
=== Round 2 ===
- Player Bob writing reset()
- Player Bob writing hit()
- Player Bob writing shake()
- Player Bob writing hit()
- Player Bob writing shake()
- Player Bob writing hit()
- Player Bob reading look()
- Player Bob sees number: 3n
=== Testing Access Control ===
- Attempting Alice's look() in Bob's round (should revert)
✅ Received expected revert
```

This output logs the events during two rounds of gameplay in the Walnut contract, showing interactions by Alice and Bob, along with a revert error when Alice attempts to call `look()`in Round 2.

Congratulations! You've reached the end of the tutorial. You can find the code for the entire project [here](https://github.com/SeismicSystems/seismic-starter/tree/ameya/baby-walnut).
