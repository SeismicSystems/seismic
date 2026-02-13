// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {AesLib} from "../lib/AesLib.sol";

import {Directory} from "../src/directory/Directory.sol";
import {Intelligence} from "../src/intelligence/Intelligence.sol";

contract IntelligenceTest is Test {
    Intelligence intelligence;
    Directory directory;
    address[] providers;

    bytes sampleMsg = "hello world";
    address alice = makeAddr("alice");
    address bob = makeAddr("bob");
    suint256 aliceKey;
    suint256 bobKey;

    function setUp() public {
        intelligence = new Intelligence();
        vm.startPrank(intelligence.INITIAL_OWNER());
        intelligence.addProvider(alice);
        intelligence.addProvider(bob);
        vm.stopPrank();

        directory = Directory(intelligence.DIRECTORY_ADDRESS());
        vm.prank(alice);
        directory.setKey(suint256(0xABC));
        vm.prank(alice);
        aliceKey = suint256(directory.getKey());
        vm.prank(bob);
        directory.setKey(suint256(0xDEF));
        vm.prank(bob);
        bobKey = suint256(directory.getKey());
    }

    function extractCT(
        bytes memory _encryptedData
    ) internal view returns (bytes memory ct) {
        (ct, ) = directory.parseEncryptedData(_encryptedData);
    }

    function testEncryptToProviders() public {
        (bytes32[] memory hashes, bytes[] memory encryptedData) = intelligence
            .encryptToProviders(sampleMsg);

        bytes[] memory directCiphertext = new bytes[](2);
        directCiphertext[0] = AesLib.AES256GCMEncrypt(aliceKey, 0, sampleMsg);
        directCiphertext[1] = AesLib.AES256GCMEncrypt(bobKey, 1, sampleMsg);
        assertEq(extractCT(encryptedData[0]), directCiphertext[0]);
        assertEq(extractCT(encryptedData[1]), directCiphertext[1]);

        bytes32[] memory directHashes = new bytes32[](2);
        directHashes[0] = keccak256(abi.encodePacked(aliceKey));
        directHashes[1] = keccak256(abi.encodePacked(bobKey));
        assertEq(hashes[0], directHashes[0]);
        assertEq(hashes[1], directHashes[1]);
    }

    function testAddProvider() public {
        address charlie = makeAddr("charlie");
        vm.prank(charlie);
        directory.setKey(suint256(0x123));
        vm.prank(charlie);
        suint256 charlieKey = suint256(directory.getKey());

        vm.prank(intelligence.owner());
        intelligence.addProvider(charlie);

        (bytes32[] memory hashes, bytes[] memory encryptedData) = intelligence
            .encryptToProviders(sampleMsg);
        bytes memory directCiphertext = AesLib.AES256GCMEncrypt(
            charlieKey,
            2,
            sampleMsg
        );

        assertEq(intelligence.numProviders(), 3);
        assertEq(extractCT(encryptedData[2]), directCiphertext);
        assertEq(hashes[2], keccak256(abi.encodePacked(charlieKey)));

        vm.prank(intelligence.owner());
        vm.expectRevert("DUPLICATE_PROVIDER");
        intelligence.addProvider(charlie);
    }

    function testRemoveProvider() public {
        vm.prank(intelligence.owner());
        intelligence.removeProvider(alice);
        assertEq(intelligence.numProviders(), 1);

        (bytes32[] memory hashes, bytes[] memory encryptedData) = intelligence
            .encryptToProviders(sampleMsg);
        assertEq(hashes.length, 1);
        assertEq(encryptedData.length, 1);

        bytes memory directCiphertext = AesLib.AES256GCMEncrypt(
            bobKey,
            0,
            sampleMsg
        );
        assertEq(extractCT(encryptedData[0]), directCiphertext);

        bytes32 h = directory.keyHash(bob);
        assertEq(h, keccak256(abi.encodePacked(bobKey)));
    }

    function test_RevertIfRemoveUnkownProvider() public {
        address unknownProvider = makeAddr("unknown");
        vm.prank(intelligence.owner());
        vm.expectRevert("PROVIDER_NOT_FOUND");
        intelligence.removeProvider(unknownProvider);
    }

    function test_RevertIfNotOwner() public {
        address charlie = makeAddr("charlie");
        vm.prank(alice);
        vm.expectRevert("UNAUTHORIZED");
        intelligence.addProvider(charlie);
    }

    function testTransferOwnership() public {
        vm.prank(intelligence.owner());
        intelligence.transferOwnership(alice);

        address charlie = makeAddr("charlie");
        vm.prank(alice);
        intelligence.addProvider(charlie);
        assertEq(intelligence.numProviders(), 3);
    }
}
