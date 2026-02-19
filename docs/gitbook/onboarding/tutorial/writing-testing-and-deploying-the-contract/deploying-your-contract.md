---
icon: hive
---

# Deploying your contract

In this chapter, you’ll deploy your Walnut contract to a local Seismic node for testing. By the end of this guide, you’ll have a fully deployed contract that you can interact with using your CLI or scripts. _Estimated Time: \~15 minutes._

### Writing the deploy script

Navigate to the script folder in your Walnut App and open the `Walnut.s.sol` file located at:

```bash
packages/contracts/script
```

and add the following to it:

```solidity
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Walnut} from "../src/Walnut.sol";

contract WalnutScript is Script {
    Walnut public walnut;

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVKEY");

        vm.startBroadcast(deployerPrivateKey);
        walnut = new Walnut(3, suint256(0));
        vm.stopBroadcast();
    }
}
```

This script will deploy a new instance of the Walnut contract with an initial shell strength of 3 and an initial kernel value of 0.

### Deploying the contract

1. In a separate terminal window, run&#x20;

```bash
sanvil
```

in order to spin up a local Seismic node.

2. In `packages/contracts` , create a `.env` file and add the following to it:

```properties
RPC_URL=http://127.0.0.1:8545
PRIVKEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

The `RPC_URL` denotes the port on which `sanvil`  is running and the `PRIVKEY` is one of the nine standard `sanvil` testing private keys.

3. Now, from `packages/contracts`, run

```bash
source .env
sforge script script/Walnut.s.sol:WalnutScript \
    --rpc-url $RPC_URL \
    --broadcast
```

Your contract should be up and deployed to your local Seismic node!
