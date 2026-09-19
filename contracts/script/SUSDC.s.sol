// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {SUSDC} from "../src/examples/SUSDC.sol";

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
