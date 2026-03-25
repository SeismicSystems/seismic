---
icon: hive
---

# Deploying

In this chapter, you'll deploy your ClownBeatdown contract to a local Seismic node for testing. By the end of this guide, you'll have a fully deployed contract that you can interact with using your CLI or scripts. _Estimated Time: ~15 minutes._

### Writing the deploy script

Navigate to the script folder in your project and open the `ClownBeatdown.s.sol` file located at:

```bash
packages/contracts/script
```

and add the following to it:

```solidity
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ClownBeatdown} from "../src/ClownBeatdown.sol";

contract ClownBeatdownScript is Script {
    ClownBeatdown public clownBeatdown;

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVKEY");

        vm.startBroadcast(deployerPrivateKey);
        clownBeatdown = new ClownBeatdown(3);
        vm.stopBroadcast();

        console.log("Deployed at:", address(clownBeatdown));
    }
}
```

This script will deploy a new instance of the ClownBeatdown contract with an initial stamina of 3. We'll add secrets separately in the next step, since `addSecret` performs shielded writes that need to be sent as on-chain transactions.

### Deploying the contract

1. In a separate terminal window, run

```bash
sanvil
```

in order to spin up a local Seismic node.

2. In `packages/contracts`, create a `.env` file and add the following to it:

```properties
RPC_URL=http://127.0.0.1:8545
PRIVKEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

The `RPC_URL` denotes the port on which `sanvil` is running and the `PRIVKEY` is one of the standard `sanvil` testing private keys.

3. Now, from `packages/contracts`, deploy the contract:

```bash
source .env
sforge script script/ClownBeatdown.s.sol:ClownBeatdownScript \
    --rpc-url $RPC_URL \
    --broadcast
```

The output will show the deployed contract address (e.g. `0x5FbDB2315678afecb367f032d93F642f64180aa3`).

4. Add secrets to the deployed contract using `scast send`. Replace `<CONTRACT_ADDRESS>` with the address from the previous step:

```bash
scast send <CONTRACT_ADDRESS> "addSecret(string)" "The moon is made of cheese" \
    --rpc-url $RPC_URL --private-key $PRIVKEY
scast send <CONTRACT_ADDRESS> "addSecret(string)" "Clowns rule the underworld" \
    --rpc-url $RPC_URL --private-key $PRIVKEY
scast send <CONTRACT_ADDRESS> "addSecret(string)" "The cake is a lie" \
    --rpc-url $RPC_URL --private-key $PRIVKEY
scast send <CONTRACT_ADDRESS> "addSecret(string)" "42 is the answer" \
    --rpc-url $RPC_URL --private-key $PRIVKEY
scast send <CONTRACT_ADDRESS> "addSecret(string)" "Never trust a smiling clown" \
    --rpc-url $RPC_URL --private-key $PRIVKEY
```

Your contract should be up and deployed to your local Seismic node with 5 secrets!
