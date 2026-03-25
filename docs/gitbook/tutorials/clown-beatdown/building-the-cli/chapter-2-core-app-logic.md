---
icon: tablet-screen-button
---

# Ch 2: Core App Logic

In this chapter, you'll write the core logic to interact with the ClownBeatdown contract by creating an App class. This class will initialize player-specific wallet clients and contracts, and provide easy-to-use functions like hit, rob, and reset. _Estimated time: ~20 minutes_

Now, navigate to `packages/cli/src/` and create a file called `app.ts` which will contain the core logic for the CLI:

```
# Assuming you are in packages/cli/lib
cd ../src
touch app.ts
```

### Import required dependencies

Start by importing all the necessary modules and functions at the top of `app.ts`:

```typescript
import {
  type ShieldedContract,
  type ShieldedWalletClient,
  createShieldedWalletClient,
} from "seismic-viem";
import { Abi, Address, Chain, http, hexToString } from "viem";
import { privateKeyToAccount } from "viem/accounts";

import { getShieldedContractWithCheck } from "../lib/utils";
```

### Define the app configuration

The `AppConfig` interface organizes all settings for the Clown Beatdown app, including player info, wallet setup, and contract details. It supports a multiplayer environment, with multiple players having distinct private keys and contract interactions.

```typescript
interface AppConfig {
  players: Array<{
    name: string; // Name of the player
    privateKey: string; // Private key for the player's wallet
  }>;
  wallet: {
    chain: Chain; // Blockchain network (e.g., Seismic Testnet or sanvil)
    rpcUrl: string; // RPC URL for blockchain communication
  };
  contract: {
    abi: Abi; // The contract's ABI for interaction
    address: Address; // The contract's deployed address
  };
}
```

### Create the App class

The `App` class manages player-specific wallet clients and contract instances, providing an easy-to-use interface for multiplayer gameplay.

```typescript
export class App {
  private config: AppConfig; // Holds all app configuration
  private playerClients: Map<string, ShieldedWalletClient> = new Map(); // Maps player names to their wallet clients
  private playerContracts: Map<string, ShieldedContract> = new Map(); // Maps player names to their contract instances

  constructor(config: AppConfig) {
    this.config = config;
  }
}
```

### Add initialization logic to App

The `init()` method sets up individual wallet clients and contract instances for each player, enabling multiplayer interactions. Each player gets their own wallet client and a direct connection to the contract.

```typescript
async init() {
  for (const player of this.config.players) {
    // Create a wallet client for the player
    const walletClient = await createShieldedWalletClient({
      chain: this.config.wallet.chain,
      transport: http(this.config.wallet.rpcUrl),
      account: privateKeyToAccount(player.privateKey as `0x${string}`),
    })
    this.playerClients.set(player.name, walletClient) // Map the client to the player

    // Initialize the player's contract instance and ensure the contract is deployed
    const contract = await getShieldedContractWithCheck(
      walletClient,
      this.config.contract.abi,
      this.config.contract.address
    )
    this.playerContracts.set(player.name, contract) // Map the contract to the player
  }
}
```

### Add helper methods to App

These helper methods ensure that the app fetches the correct contract instance for a specific player, supporting multiplayer scenarios.

`getPlayerContract`:

```typescript
private getPlayerContract(playerName: string): ShieldedContract {
  const contract = this.playerContracts.get(playerName)
  if (!contract) {
    throw new Error(`Shielded contract for player ${playerName} not found`)
  }
  return contract
}
```

### Implement Contract Interaction Methods

`reset`

Resets the clown for the next round. The reset restores stamina and picks a new random secret.

```typescript
async reset(playerName: string) {
  console.log(`- Player ${playerName} writing reset()`)
  const contract = this.getPlayerContract(playerName)
  await contract.write.reset([])
}
```

`hit`

A player can hit the clown to reduce its stamina. Each hit is logged for the respective player.

```typescript
async hit(playerName: string) {
  console.log(`- Player ${playerName} writing hit()`)
  const contract = this.getPlayerContract(playerName)
  await contract.write.hit([])
}
```

`rob`

Reveals a secret for a specific player if they contributed to knocking out the clown. This ensures fairness in multiplayer gameplay. Uses **signed reads.**

```typescript
async rob(playerName: string) {
  console.log(`- Player ${playerName} reading rob()`)
  const contract = this.getPlayerContract(playerName)
  const result = await contract.read.rob() // signed read
  const decoded = hexToString(result as `0x${string}`)
  console.log(`- Player ${playerName} robbed secret:`, decoded)
}
```
