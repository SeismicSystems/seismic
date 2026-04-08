---
icon: link
---

# Ch 2: Contract Hooks

In this chapter, you'll build the React hooks that connect your UI to the ClownBeatdown contract. These hooks encapsulate all contract interaction logic so your components stay clean. _Estimated time: ~15 minutes_

### Contract ABI setup

First, copy the compiled ABI from your contracts build output into the web package. After deploying (see [Deploying](../writing-the-contract/deploying.md)), copy the ABI file:

```bash
mkdir -p packages/web/src/abis/contracts
cp packages/contracts/out/ClownBeatdown.sol/ClownBeatdown.json \
   packages/web/src/abis/contracts/ClownBeatdown.json
```

You'll also need to add the deployed contract address and chain ID to the JSON file. After copying, edit the file to include `address` and `chainId` at the top level. The final structure should look like:

```json
{
  "address": "0xYourDeployedAddress",
  "chainId": 31337,
  "abi": [
    { "type": "constructor", "inputs": [...] },
    { "type": "function", "name": "hit", ... },
    ...
  ]
}
```

You can find the deployed address in `packages/contracts/broadcast/ClownBeatdown.s.sol/31337/run-latest.json` under `transactions[0].contractAddress`.

### Contract type definition

Create `src/types/contract.ts`:

```typescript
export type ContractInterface = {
  chainId: number;
  abi: Array<Record<string, unknown>>;
  methodIdentifiers: Record<string, string>;
};

export type DeployedContract = ContractInterface & {
  address: `0x${string}`;
};
```

### useContract hook

This hook creates a shielded contract instance using `seismic-react`. Create `src/hooks/useContract.ts`:

```typescript
import { useShieldedContract } from "seismic-react";

import * as contractJson from "@/abis/contracts/ClownBeatdown.json" with { type: "json" };
import type { DeployedContract } from "@/types/contract";

export const useAppContract = () =>
  useShieldedContract(contractJson as DeployedContract);
```

The `useShieldedContract` hook from `seismic-react` returns a contract instance that supports both shielded writes and signed reads — the same interface you used in the CLI with `getShieldedContract`, but integrated with React's lifecycle.

### useContractClient hook

This hook wraps the contract methods into callable functions with proper error handling. Create `src/hooks/useContractClient.ts`:

```typescript
import { useCallback, useEffect, useState } from "react";
import { useShieldedWallet } from "seismic-react";
import {
  type ShieldedPublicClient,
  type ShieldedWalletClient,
  addressExplorerUrl,
  txExplorerUrl,
} from "seismic-viem";
import { type Hex, hexToString } from "viem";

import { useAppContract } from "./useContract";

export const useContractClient = () => {
  const [loaded, setLoaded] = useState(false);
  const { walletClient, publicClient } = useShieldedWallet();
  const { contract } = useAppContract();

  useEffect(() => {
    if (walletClient && publicClient && contract) {
      setLoaded(true);
    } else {
      setLoaded(false);
    }
  }, [walletClient, publicClient, contract]);

  const wallet = useCallback((): ShieldedWalletClient => {
    if (!walletClient) {
      throw new Error("Wallet client not found");
    }
    return walletClient;
  }, [walletClient]);

  const pubClient = useCallback((): ShieldedPublicClient => {
    if (!publicClient) {
      throw new Error("Public client not found");
    }
    return publicClient;
  }, [publicClient]);

  const walletAddress = useCallback((): Hex => {
    return wallet().account.address;
  }, [wallet]);

  const appContract = useCallback((): ReturnType<
    typeof useAppContract
  >["contract"] => {
    if (!contract) {
      throw new Error("Contract not found");
    }
    return contract;
  }, [contract]);

  /*
    function getClownStamina() external view returns (uint256);
    function rob() external view returns (bytes32);
    function hit() external;
    function reset() external;
  */

  const clownStamina = useCallback(async (): Promise<bigint> => {
    return appContract().tread.getClownStamina();
  }, [appContract]);

  const rob = useCallback(async (): Promise<string> => {
    const result = (await appContract().read.rob()) as Hex;
    return hexToString(result);
  }, [appContract]);

  const hit = useCallback(async (): Promise<Hex> => {
    return appContract().twrite.hit();
  }, [appContract]);

  const reset = useCallback(async (): Promise<Hex> => {
    return appContract().twrite.reset();
  }, [appContract]);

  const txUrl = useCallback(
    (txHash: Hex): string | null => {
      return txExplorerUrl({ chain: pubClient().chain, txHash });
    },
    [pubClient],
  );

  const addressUrl = useCallback(
    (address: Hex): string | null => {
      return addressExplorerUrl({ chain: pubClient().chain, address });
    },
    [pubClient],
  );

  const waitForTransaction = useCallback(
    async (hash: Hex) => {
      return await pubClient().waitForTransactionReceipt({ hash });
    },
    [pubClient],
  );

  return {
    loaded,
    walletClient,
    publicClient,
    walletAddress,
    appContract,
    pubClient,
    wallet,
    clownStamina,
    rob,
    hit,
    reset,
    txUrl,
    addressUrl,
    waitForTransaction,
  };
};
```

### What's happening here?

Notice the different contract namespaces used for each method:

- **`appContract().twrite.hit()`** and **`appContract().twrite.reset()`** — these are **shielded write** transactions. The `twrite` namespace sends a Seismic transaction (type 0x70) that encrypts calldata.
- **`appContract().read.rob()`** — this is a **signed read**. The `read` namespace performs a `signed_call` that proves the caller's identity to the contract, allowing `onlyContributor` to verify access. The result comes back as `Hex` and is decoded with `hexToString()`.
- **`appContract().tread.getClownStamina()`** — this is a **transparent read**. The `tread` namespace performs a standard `eth_call` since stamina is public state.

This distinction between `twrite`, `read`, and `tread` is the key difference from a standard Ethereum dApp.

### Supporting components

Before building the game actions hook, create the helper components and hooks it depends on.

**Explorer toast** — Create `src/components/chain/ExplorerToast.tsx`:

```typescript
import React from 'react'

type ExplorerToastProps = {
  url: string
  text: string
  hash: string
}

export const ExplorerToast: React.FC<ExplorerToastProps> = ({ url, text, hash }) => (
  <a href={url} target="_blank" rel="noopener noreferrer">
    {text}{hash.slice(0, 10)}...
  </a>
)
```

**Toast notifications** — Create `src/hooks/useToastNotifications.ts`:

```typescript
import { toast } from 'react-toastify'

export const useToastNotifications = () => ({
  notifySuccess: (msg: string) => toast.success(msg),
  notifyError: (msg: string) => toast.error(msg),
  notifyInfo: (msg: string | React.ReactElement) => toast.info(msg),
})
```

### useGameActions hook

This hook orchestrates the game logic, managing state and coordinating contract calls with UI feedback. Create `src/hooks/useGameActions.ts`:

```typescript
import { useCallback, useEffect, useState } from "react";
import React from "react";
import { useSound } from "use-sound";

import { ExplorerToast } from "@/components/chain/ExplorerToast";
import { useContractClient } from "@/hooks/useContractClient";
import { useToastNotifications } from "@/hooks/useToastNotifications";

export const useGameActions = () => {
  const [clownStamina, setClownStamina] = useState<number | null>(null);
  const [currentRoundId] = useState<number | null>(1);

  const {
    loaded,
    hit,
    rob,
    reset,
    txUrl,
    waitForTransaction,
    clownStamina: readClownStamina,
  } = useContractClient();

  const { notifySuccess, notifyError, notifyInfo } = useToastNotifications();
  const [isHitting, setIsHitting] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [isRobbing, setIsRobbing] = useState(false);
  const [robResult, setRobResult] = useState<string | null>(null);
  const [punchCount, setPunchCount] = useState(0);
  const [playHit] = useSound("/audio/hit_sfx.wav", { volume: 0.1 });
  const [playReset] = useSound("/audio/reset_sfx.wav", { volume: 0.1 });
  const [playRob] = useSound("/audio/rob_sfx.wav", { volume: 0.1 });

  const fetchGameRounds = useCallback(() => {
    if (!loaded) return;
    readClownStamina()
      .then((stamina) => {
        setClownStamina(Number(stamina));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        console.error("Error fetching clown stamina:", message);
      });
  }, [loaded, readClownStamina]);

  // Fetch initial state when contract is loaded
  useEffect(() => {
    fetchGameRounds();
  }, [fetchGameRounds]);

  const resetGameState = useCallback(() => {
    setRobResult(null);
    setPunchCount(0);
  }, [punchCount]);

  const handleHit = async () => {
    playHit();
    if (!loaded || isHitting) return;
    setIsHitting(true);
    hit()
      .then((hash) => {
        const url = txUrl(hash);
        if (url) {
          notifyInfo(
            React.createElement(ExplorerToast, {
              url: url,
              text: "Sent punch tx: ",
              hash: hash,
            }),
          );
        } else {
          notifyInfo(`Sent punch tx: ${hash}`);
        }
        if (clownStamina && clownStamina > 0) {
          setPunchCount((prev) => {
            const newCount = Math.min(prev + 1, 3);
            return newCount;
          });
        }
        return waitForTransaction(hash);
      })
      .then((receipt) => {
        if (receipt.status === "success") {
          notifySuccess("Punch successful");
          // Re-read stamina from contract after successful hit
          fetchGameRounds();
        } else {
          notifyError("Punch failed");
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        notifyError(`Error punching clown: ${message}`);
      })
      .finally(() => {
        setIsHitting(false);
      });
  };

  const handleReset = async () => {
    playReset();
    if (!loaded || isResetting) return;
    if (clownStamina !== 0) {
      notifyError("Clown must be KO to reset");
      return;
    }
    setIsResetting(true);
    reset()
      .then((hash) => {
        const url = txUrl(hash);
        if (url) {
          notifyInfo(
            React.createElement(ExplorerToast, {
              url: url,
              text: "Sent reset tx: ",
              hash: hash,
            }),
          );
        } else {
          notifyInfo(`Sent reset tx: ${hash}`);
        }
        setPunchCount(0);
        return waitForTransaction(hash);
      })
      .then((receipt) => {
        if (receipt.status === "success") {
          notifySuccess("Reset successful");
          setRobResult(null);
          // Re-read stamina from contract after successful reset
          fetchGameRounds();
        } else {
          notifyError("Reset failed");
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        notifyError(`Error resetting clown: ${message}`);
      })
      .finally(() => {
        setIsResetting(false);
      });
  };

  const handleRob = async () => {
    playRob();
    if (!loaded || isRobbing) return;
    setIsRobbing(true);
    rob()
      .then((result) => {
        setRobResult(result);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        notifyError(`Error robbing clown: ${message}`);
      })
      .finally(() => {
        setIsRobbing(false);
      });
  };

  return {
    loaded,
    clownStamina,
    currentRoundId,
    isHitting,
    isResetting,
    isRobbing,
    robResult,
    punchCount,
    fetchGameRounds,
    resetGameState,
    handleHit,
    handleReset,
    handleRob,
  };
};
```

This hook manages the full game lifecycle:

- **`fetchGameRounds`** — reads the current stamina from the contract via `tread.getClownStamina()`
- **`handleHit`** — sends a shielded write via `twrite.hit()`, shows toast notifications with explorer links, waits for the receipt, increments punch count, and refetches stamina
- **`handleReset`** — validates the clown is KO, sends a shielded write via `twrite.reset()`, clears punch count and rob result, and refetches stamina
- **`handleRob`** — performs a signed read via `read.rob()` to decrypt and reveal a secret from the clown's pool
- **`resetGameState`** — clears the rob result and punch count when the round changes
