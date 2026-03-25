---
icon: people
---

# Ch 3: Bringing It All Together

Now that we've built the core logic for interacting with the ClownBeatdown contract, it's time to tie everything together into a CLI that runs a multiplayer game session with two players — **Alice and Bob.** In this chapter, you'll set up the environment variables for multiple players and write `index.ts` to simulate gameplay. _Estimated time: ~20 minutes_

### Set Up Environment Variables

Before running the game, we need to define environment variables that store important configurations such as the RPC URL, chain ID, and player private keys.

Create a `.env` in `packages/cli`:

```bash
touch .env
```

Open `.env` and paste the following:

```properties
CHAIN_ID=31337
VITE_CHAIN_ID=31337
RPC_URL=http://127.0.0.1:8545
ALICE_PRIVKEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
BOB_PRIVKEY=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
```

**What's Happening Here?**

- `CHAIN_ID=31337`: Used to locate the deployment broadcast file for the correct chain.
- `VITE_CHAIN_ID=31337`: Used for chain selection (`sanvil` vs testnet). `31337` is the default chain ID for `sanvil` (your local Seismic node).
- `RPC_URL=http://127.0.0.1:8545`: This is the RPC URL for interacting with the local Seismic node.
- `ALICE_PRIVKEY` and `BOB_PRIVKEY`: These are Alice and Bob's private keys, allowing them to play the game. (These are standard test keys provided by `sanvil`)

### Write index.ts

Now, we'll create the main entry point for our game session. This file will simulate gameplay, initializing players and having them interact with the ClownBeatdown contract. Open `packages/cli/src/index.ts` and follow these steps:

#### Import Dependencies

Import the required libraries to read environment variables, define network configurations, and interact with the ClownBeatdown contract.

```typescript
import dotenv from "dotenv";
import { join } from "path";
import { sanvil, seismicTestnet } from "seismic-viem";

import { CONTRACT_DIR, CONTRACT_NAME } from "../lib/constants";
import { readContractABI, readContractAddress } from "../lib/utils";
import { App } from "./app";

// Load environment variables from .env file
dotenv.config();
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

The contract's ABI and deployed address are read from files generated during deployment.

```typescript
const broadcastFile = join(
  CONTRACT_DIR,
  "broadcast",
  `${CONTRACT_NAME}.s.sol`,
  process.env.CHAIN_ID,
  "run-latest.json",
);
const abiFile = join(
  CONTRACT_DIR,
  "out",
  `${CONTRACT_NAME}.sol`,
  `${CONTRACT_NAME}.json`,
);
```

#### Select the blockchain network

Determine whether to use the local `sanvil` node (31337) or the Seismic testnet.

```typescript
const chain =
  process.env.VITE_CHAIN_ID === sanvil.id.toString() ? sanvil : seismicTestnet;
```

#### Define players

Assign Alice and Bob as players with private keys stored in `.env`.

```typescript
const players = [
  { name: "Alice", privateKey: process.env.ALICE_PRIVKEY! },
  { name: "Bob", privateKey: process.env.BOB_PRIVKEY! },
];
```

#### Initialize the Game App

Create an `App` instance to interact with the ClownBeatdown contract.

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
});

await app.init();
```

#### Simulate the game round by round

The following logic executes two rounds of gameplay between Alice and Bob.

**Round 1 — Alice Plays**

Alice hits the clown three times (stamina starts at 3), knocking it out, then robs a secret.

```typescript
console.log("=== Round 1 ===");
await app.hit("Alice");
await app.hit("Alice");
await app.hit("Alice");

// Alice robs the clown's secret in round 1
await app.rob("Alice");
```

**Round 2 — Bob Plays**

Bob resets the clown, hits it three more times, then robs his own secret.

```typescript
console.log("=== Round 2 ===");
await app.reset("Bob");
await app.hit("Bob");
await app.hit("Bob");
await app.hit("Bob");

// Bob robs the clown's secret in round 2
await app.rob("Bob");
```

**Alice Tries to Rob in Round 2 (we expect this to fail since she contributed in round 1 but not round 2)**

```typescript
  // Alice tries to rob in round 2, should fail by reverting
  console.log('=== Testing Access Control ===')
  console.log("Attempting Alice's rob() in Bob's round (should revert)")
  try {
    await app.rob('Alice')
    console.error('Expected rob() to revert but it succeeded')
    process.exit(1)
  } catch (error) {
    console.log('Received expected revert')
  }
}
```

### Execute the main() function

This ensures that the script runs when executed.

```typescript
main();
```

### Running the CLI

Now, run the CLI from `packages/cli` by running:

```bash
bun dev
```

You should see something like this as the output:

```
=== Round 1 ===
- Player Alice writing hit()
- Player Alice writing hit()
- Player Alice writing hit()
- Player Alice reading rob()
- Player Alice robbed secret: The cake is a lie
=== Round 2 ===
- Player Bob writing reset()
- Player Bob writing hit()
- Player Bob writing hit()
- Player Bob writing hit()
- Player Bob reading rob()
- Player Bob robbed secret: 42 is the answer
=== Testing Access Control ===
Attempting Alice's rob() in Bob's round (should revert)
Received expected revert
```

This output logs the events during two rounds of gameplay in the ClownBeatdown contract, showing interactions by Alice and Bob, along with a revert error when Alice attempts to call `rob()` in Round 2.

Congratulations! You've reached the end of the tutorial.
