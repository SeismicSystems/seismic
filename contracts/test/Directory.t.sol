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

    function test_CheckHasKeyReturnsFalseForUnsetAddress() public {
        address carol = makeAddr("carol");
        assertFalse(directory.checkHasKey(carol));
    }

    function test_CheckHasKeyReturnsTrueAfterSetKey() public {
        address carol = makeAddr("carol");
        vm.prank(carol);
        directory.setKey(suint256(0x123));

        assertTrue(directory.checkHasKey(carol));
    }

    function test_PackEncryptedDataAppendsTwelveByteNonce() public view {
        bytes memory ciphertext = hex"deadbeefcafe";
        uint96 nonce = 0x0102030405060708090a0b0c;

        bytes memory packed = directory.packEncryptedData(ciphertext, nonce);

        assertEq(packed.length, ciphertext.length + 12);
    }

    function test_PackEncryptedDataRoundTripsThroughParse() public view {
        bytes memory ciphertext = hex"00112233445566778899aabbccddeeff0011223344";
        uint96 nonce = type(uint96).max - 7;

        bytes memory packed = directory.packEncryptedData(ciphertext, nonce);
        (bytes memory parsedCiphertext, uint96 parsedNonce) = directory.parseEncryptedData(packed);

        assertEq(parsedCiphertext, ciphertext);
        assertEq(parsedNonce, nonce);
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
