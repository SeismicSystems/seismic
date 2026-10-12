// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {SUSDC} from "../src/examples/SUSDC.sol";
import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

/// @notice Deploys SUSDC directly (constructor sets admin = broadcaster).
contract SUSDCScript is Script {
    SUSDC public token;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        token = new SUSDC();

        console.log("SUSDC deployed at:", address(token));
        console.log("Admin:", token.admin());

        vm.stopBroadcast();
    }
}

/// @notice Deploys SUSDC behind a TransparentUpgradeableProxy.
/// @dev Env:
///   SUSDC_ADMIN        token admin (mint/burn/pause/setBridge); defaults to broadcaster
///   SUSDC_PROXY_OWNER  owner of the auto-created ProxyAdmin (upgrade authority); defaults to broadcaster
///   SUSDC_BRIDGE       optional bridge router to set immediately (e.g. Hyperlane HypXERC20)
contract SUSDCProxyScript is Script {
    SUSDC public implementation;
    SUSDC public token; // proxy viewed through the SUSDC ABI

    function setUp() public {}

    function run() public {
        (, address broadcaster,) = vm.readCallers();
        address admin = vm.envOr("SUSDC_ADMIN", broadcaster);
        address proxyOwner = vm.envOr("SUSDC_PROXY_OWNER", broadcaster);
        address bridge = vm.envOr("SUSDC_BRIDGE", address(0));

        vm.startBroadcast();

        implementation = new SUSDC();
        TransparentUpgradeableProxy proxy = new TransparentUpgradeableProxy(
            address(implementation), proxyOwner, abi.encodeCall(SUSDC.initialize, (admin))
        );
        token = SUSDC(address(proxy));

        if (bridge != address(0)) {
            require(admin == broadcaster, "SUSDC_BRIDGE requires broadcaster to be admin");
            token.setBridge(bridge);
        }

        vm.stopBroadcast();

        console.log("SUSDC implementation:", address(implementation));
        console.log("SUSDC proxy (use this):", address(token));
        console.log("Admin:", token.admin());
        console.log("ProxyAdmin owner:", proxyOwner);
        console.log("Bridge:", token.bridge());
    }
}
