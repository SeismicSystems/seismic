// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import {MockSRC20} from "./utils/MockSRC20.sol";
import {Directory} from "../src/directory/Directory.sol";
import {Intelligence} from "../src/intelligence/Intelligence.sol";
import {IDirectory} from "seismic-std-lib/interfaces/IDirectory.sol";
import {CryptoUtils} from "seismic-std-lib/utils/precompiles/CryptoUtils.sol";

/// @dev Attacker helper: captures ciphertext C1 for a chosen plaintext (32 zero bytes) under the
///      VICTIM's key at the current Directory nonce. When the nonce is a shared counter, the inner
///      revert rolls back Directory's `nonce++`, so C1 is produced at the exact nonce the victim's
///      next `encrypt` will reuse. An `eth_call` would freeze the counter just the same.
contract Encrypter {
    Directory private directory;

    constructor(Directory _directory) {
        directory = _directory;
    }

    function encryptRevert(address victim) public {
        bytes memory encrypted = directory.encrypt(victim, abi.encodePacked(bytes32(uint256(0))));
        assembly {
            revert(add(encrypted, 32), mload(encrypted))
        }
    }

    function encrypt(address victim) public returns (bytes memory) {
        (, bytes memory data) = address(this).call(abi.encodeWithSelector(this.encryptRevert.selector, victim));
        return data; // ciphertext returned via revert data
    }
}

/// @notice Regression test for Zellic seismic-std-lib finding 3.1 — AES-GCM nonce reuse.
///         Asserts the SECURE property: each `Directory.encrypt` call must draw a fresh nonce, so an
///         attacker who freezes a shared counter (via a reverting call or `eth_call`) cannot recover
///         a victim's shielded SRC20 transfer amount from `C1 XOR C2`.
contract DirectoryNonceReuseTest is Test {
    address constant DIRECTORY_ADDR = 0x1000000000000000000000000000000000000004;
    address constant INTELLIGENCE_ADDR = 0x1000000000000000000000000000000000000005;
    bytes32 constant TRANSFER_TOPIC = keccak256("Transfer(address,address,bytes32,bytes)");

    IDirectory directory = IDirectory(DIRECTORY_ADDR);
    address alice = makeAddr("alice"); // victim
    address bob = makeAddr("bob"); // transfer recipient

    MockSRC20 token;
    Encrypter encrypter;

    function setUp() public {
        deployCodeTo("Directory.sol", DIRECTORY_ADDR);
        deployCodeTo("Intelligence.sol", INTELLIGENCE_ADDR);

        token = new MockSRC20("Token", "TKN", 18);
        encrypter = new Encrypter(Directory(DIRECTORY_ADDR));

        // Alice registers a shielded AES key in the Directory.
        suint256 aliceKey = CryptoUtils.generateRandomAESKey();
        vm.prank(alice);
        directory.setKey(aliceKey);

        token.mint(alice, suint256(1000e18));
    }

    /// A fresh per-call nonce defeats the freeze-and-XOR recovery: the attacker's C1 and the
    /// victim's C2 encrypt under the same key but different nonces, so their keystreams no longer
    /// cancel and `C1 XOR C2` does not reveal the amount. On the vulnerable counter-based code both
    /// ciphertexts share nonce N and this test fails.
    function test_NonceReuseViaRevertDoesNotLeakAmount() public {
        // (1) Attacker captures C1 = encrypt(alice, 32 zero bytes) without committing state.
        bytes memory c1 = encrypter.encrypt(alice);

        // (2) Alice transfers to Bob -> SRC20 emits C2 = encrypt(alice, amount) in a Transfer log.
        uint256 transferAmount = 1000e18;
        vm.recordLogs();
        vm.prank(alice);
        token.transfer(bob, suint256(transferAmount));

        // (3) Attacker reads C2 from the public Transfer log (matched by keyHash(alice)).
        bytes memory c2 = _extractEncryptedAmount(alice);

        assertTrue(c1.length > 0 && c2.length > 0, "ciphertexts must be non-empty");
        assertEq(c1.length, c2.length, "same plaintext size => same ciphertext length");

        // SECURE property 1: the packed nonces (trailing 12 bytes) must differ across calls.
        assertTrue(_packedNonce(c1) != _packedNonce(c2), "each encrypt must draw a fresh nonce");

        // SECURE property 2: without a shared nonce, C1 XOR C2 must not equal the amount.
        uint256 recovered = _ciphertextBody(c1) ^ _ciphertextBody(c2);
        assertTrue(recovered != transferAmount, "keystream reuse must not leak the shielded amount");
    }

    /// @dev First 32 bytes = the GCM ciphertext body (plaintext XOR keystream).
    function _ciphertextBody(bytes memory data) private pure returns (uint256) {
        bytes32 b32;
        assembly {
            b32 := mload(add(data, 32))
        }
        return uint256(b32);
    }

    /// @dev Last 12 bytes = the nonce packed by `Directory.packEncryptedData`.
    function _packedNonce(bytes memory data) private pure returns (uint96) {
        uint256 start = data.length - 12;
        bytes memory nonceBytes = new bytes(12);
        for (uint256 i = 0; i < 12; i++) {
            nonceBytes[i] = data[start + i];
        }
        return uint96(bytes12(nonceBytes));
    }

    function _extractEncryptedAmount(address keyOwner) internal returns (bytes memory) {
        bytes32 ownerKeyHash = directory.keyHash(keyOwner);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == TRANSFER_TOPIC && logs[i].topics[3] == ownerKeyHash) {
                return abi.decode(logs[i].data, (bytes));
            }
        }
        revert("Transfer event for keyOwner not found");
    }
}
