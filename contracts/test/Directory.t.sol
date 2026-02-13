// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {Directory} from "../src/directory/Directory.sol";

contract DirectoryTest is Test {
    Directory directory;

    bytes sampleMsg = "hello world";
    address alice = makeAddr("alice");
    address bob = makeAddr("bob");

    function setUp() public {
        directory = new Directory();

        vm.prank(alice);
        directory.setKey(suint256(0xABC));
        vm.prank(bob);
        directory.setKey(suint256(0xDEF));
    }

    function testEncrypt() public {
        bytes memory encryptedData = directory.encrypt(alice, sampleMsg);
        vm.prank(alice);
        bytes memory decryptResult = directory.decrypt(encryptedData);

        assertEq(decryptResult, sampleMsg);
    }

    function testEncryptSequence() public {
        bytes memory encryptedDataBob = directory.encrypt(bob, sampleMsg);
        bytes memory encryptedDataAlice = directory.encrypt(alice, sampleMsg);

        vm.prank(bob);
        bytes memory decryptResultBob = directory.decrypt(encryptedDataBob);
        vm.prank(alice);
        bytes memory decryptResultAlice = directory.decrypt(encryptedDataAlice);

        assertEq(decryptResultBob, sampleMsg);
        assertEq(decryptResultAlice, sampleMsg);
    }
}
