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

You'll also need to add the deployed contract address and chain ID. Create a wrapper that exports the ABI with deployment info:

```typescript
// src/abis/contracts/ClownBeatdown.json
// This file contains the ABI output from sforge build,
// plus the deployed address and chainId fields
```

### useContract hook

This hook creates a shielded contract instance using `seismic-react`. Create `src/hooks/useContract.ts`:

```typescript
import { useShieldedContract } from "seismic-react";

import type { DeployedContract } from "@/types/contract";
import ClownBeatdownABI from "@/abis/contracts/ClownBeatdown.json";

const deployedContract = ClownBeatdownABI as unknown as DeployedContract;

export function useAppContract() {
  return useShieldedContract(deployedContract);
}
```

The `useShieldedContract` hook from `seismic-react` returns a contract instance that supports both shielded writes and signed reads — the same interface you used in the CLI with `getShieldedContract`, but integrated with React's lifecycle.

### useContractClient hook

This hook wraps the contract methods into callable functions with proper error handling. Create `src/hooks/useContractClient.ts`:

```typescript
import { useCallback } from "react";
import { useShieldedWallet } from "seismic-react";

import { useAppContract } from "./useContract";

export function useContractClient() {
  const { walletClient, publicClient } = useShieldedWallet();
  const { contract } = useAppContract();

  const loaded = !!(walletClient && publicClient && contract);

  const readClownStamina = useCallback(async (): Promise<bigint> => {
    return (await contract!.tread.getClownStamina()) as bigint;
  }, [contract]);

  const hit = useCallback(async (): Promise<string> => {
    return (await contract!.twrite.hit()) as string;
  }, [contract]);

  const reset = useCallback(async (): Promise<string> => {
    return (await contract!.twrite.reset()) as string;
  }, [contract]);

  const rob = useCallback(async (): Promise<string> => {
    return (await contract!.read.rob()) as string;
  }, [contract]);

  const waitForTransaction = useCallback(
    async (hash: string) => {
      return publicClient!.waitForTransactionReceipt({
        hash: hash as `0x${string}`,
      });
    },
    [publicClient],
  );

  return {
    loaded,
    readClownStamina,
    hit,
    reset,
    rob,
    waitForTransaction,
  };
}
```

### What's happening here?

Notice the different contract namespaces used for each method:

- **`contract.twrite.hit()`** and **`contract.twrite.reset()`** — these are **shielded write** transactions. The `twrite` namespace sends a Seismic transaction (type 0x70) that encrypts calldata.
- **`contract.read.rob()`** — this is a **signed read**. The `read` namespace performs a `signed_call` that proves the caller's identity to the contract, allowing `onlyContributor` to verify access.
- **`contract.tread.getClownStamina()`** — this is a **transparent read**. The `tread` namespace performs a standard `eth_call` since stamina is public state.

This distinction between `twrite`, `read`, and `tread` is the key difference from a standard Ethereum dApp.

### useGameActions hook

This hook orchestrates the game logic, managing state and coordinating contract calls with UI feedback. Create `src/hooks/useGameActions.ts`:

```typescript
import { useCallback, useEffect, useState } from "react";
import useSound from "use-sound";

import { useContractClient } from "./useContractClient";

export function useGameActions() {
  const { loaded, readClownStamina, hit, reset, rob, waitForTransaction } =
    useContractClient();

  const [clownStamina, setClownStamina] = useState<number | null>(null);
  const [isHitting, setIsHitting] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [isRobbing, setIsRobbing] = useState(false);
  const [robResult, setRobResult] = useState<string | null>(null);
  const [punchCount, setPunchCount] = useState(0);

  const [playHitSound] = useSound("/hit_sfx.wav", { volume: 0.1 });
  const [playResetSound] = useSound("/reset_sfx.wav");
  const [playRobSound] = useSound("/rob_sfx.wav");

  // Fetch stamina on load and after actions
  const fetchStamina = useCallback(async () => {
    if (!loaded) return;
    const stamina = await readClownStamina();
    setClownStamina(Number(stamina));
  }, [loaded, readClownStamina]);

  useEffect(() => {
    fetchStamina();
  }, [fetchStamina]);

  const handleHit = useCallback(async () => {
    setIsHitting(true);
    try {
      playHitSound();
      const txHash = await hit();
      await waitForTransaction(txHash);
      setPunchCount((prev) => Math.min(prev + 1, 3));
      await fetchStamina();
    } finally {
      setIsHitting(false);
    }
  }, [hit, waitForTransaction, fetchStamina, playHitSound]);

  const handleReset = useCallback(async () => {
    setIsResetting(true);
    try {
      playResetSound();
      const txHash = await reset();
      await waitForTransaction(txHash);
      setPunchCount(0);
      setRobResult(null);
      await fetchStamina();
    } finally {
      setIsResetting(false);
    }
  }, [reset, waitForTransaction, fetchStamina, playResetSound]);

  const handleRob = useCallback(async () => {
    setIsRobbing(true);
    try {
      playRobSound();
      const result = await rob();
      setRobResult(result);
    } finally {
      setIsRobbing(false);
    }
  }, [rob, playRobSound]);

  return {
    loaded,
    clownStamina,
    isHitting,
    isResetting,
    isRobbing,
    robResult,
    punchCount,
    handleHit,
    handleReset,
    handleRob,
    setRobResult,
  };
}
```

This hook manages the full game lifecycle:

- **`handleHit`** — sends a shielded write, waits for the transaction receipt, increments the punch count (for sprite animation), and refetches stamina
- **`handleReset`** — sends a shielded write to reset the clown, clears the punch count and rob result, and refetches stamina
- **`handleRob`** — performs a signed read to decrypt and reveal a secret from the clown's pool
