// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import {MockSRC20} from "./utils/MockSRC20.sol";
import {Directory} from "../src/directory/Directory.sol";
import {Intelligence} from "../src/intelligence/Intelligence.sol";
import {IDirectory} from "seismic-std-lib/interfaces/IDirectory.sol";
import {CryptoUtils} from "seismic-std-lib/utils/precompiles/CryptoUtils.sol";

/// @notice Tests that SRC20 Transfer events are emitted symmetrically
///         to both sender and recipient, regardless of whether each party
///         has a registered AES key.
contract SRC20EventsTest is Test {
    MockSRC20 token;

    address constant INTELLIGENCE_ADDR = 0x1000000000000000000000000000000000000005;
    address constant DIRECTORY_ADDR = 0x1000000000000000000000000000000000000004;

    IDirectory directory = IDirectory(DIRECTORY_ADDR);

    address sender = makeAddr("sender");
    address recipient = makeAddr("recipient");

    bytes32 constant TRANSFER_TOPIC = keccak256("Transfer(address,address,bytes32,bytes)");

    function setUp() public {
        // Deploy real Directory and Intelligence at their genesis addresses
        deployCodeTo("Directory.sol", DIRECTORY_ADDR);
        deployCodeTo("Intelligence.sol", INTELLIGENCE_ADDR);

        token = new MockSRC20("Token", "TKN", 18);
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: BOTH PARTIES HAVE KEYS
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsEventToBothPartiesWhenBothHaveKeys() public {
        _registerKey(sender);
        _registerKey(recipient);

        bytes32 senderKeyHash = directory.keyHash(sender);
        bytes32 recipientKeyHash = directory.keyHash(recipient);

        token.mint(sender, suint256(1e18));

        vm.recordLogs();
        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, senderKeyHash);
        _assertTransferLog(transferLogs[1], sender, recipient, recipientKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[0]);
        _assertEncryptedDataNonEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: ONLY SENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsEncryptedEventToSenderWhenOnlySenderHasKey() public {
        _registerKey(sender);

        bytes32 senderKeyHash = directory.keyHash(sender);

        token.mint(sender, suint256(1e18));

        vm.recordLogs();
        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, senderKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: ONLY RECIPIENT HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsEncryptedEventToRecipientWhenOnlyRecipientHasKey() public {
        _registerKey(recipient);

        bytes32 recipientKeyHash = directory.keyHash(recipient);

        token.mint(sender, suint256(1e18));

        vm.recordLogs();
        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], sender, recipient, recipientKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: NEITHER PARTY HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsZeroHashEventsToBothPartiesWhenNeitherHasKey() public {
        token.mint(sender, suint256(1e18));

        vm.recordLogs();
        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: BOTH PARTIES HAVE KEYS
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsEventToBothPartiesWhenBothHaveKeys() public {
        _registerKey(sender);
        _registerKey(recipient);

        bytes32 senderKeyHash = directory.keyHash(sender);
        bytes32 recipientKeyHash = directory.keyHash(recipient);

        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        vm.recordLogs();
        token.transferFrom(sender, recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, senderKeyHash);
        _assertTransferLog(transferLogs[1], sender, recipient, recipientKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[0]);
        _assertEncryptedDataNonEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: ONLY SENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsEncryptedEventToSenderWhenOnlySenderHasKey() public {
        _registerKey(sender);

        bytes32 senderKeyHash = directory.keyHash(sender);

        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        vm.recordLogs();
        token.transferFrom(sender, recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, senderKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: ONLY RECIPIENT HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsEncryptedEventToRecipientWhenOnlyRecipientHasKey() public {
        _registerKey(recipient);

        bytes32 recipientKeyHash = directory.keyHash(recipient);

        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        vm.recordLogs();
        token.transferFrom(sender, recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], sender, recipient, recipientKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: NEITHER PARTY HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsZeroHashEventsToBothPartiesWhenNeitherHasKey() public {
        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        vm.recordLogs();
        token.transferFrom(sender, recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], sender, recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    MINT: RECIPIENT HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_MintEmitsEncryptedEventToRecipientWhenRecipientHasKey() public {
        _registerKey(recipient);

        bytes32 recipientKeyHash = directory.keyHash(recipient);

        vm.recordLogs();
        token.mint(recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        // address(0) has no key → zero hash
        _assertTransferLog(transferLogs[0], address(0), recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[0]);
        // recipient has key → encrypted
        _assertTransferLog(transferLogs[1], address(0), recipient, recipientKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    MINT: RECIPIENT HAS NO KEY
    //////////////////////////////////////////////////////////////*/

    function test_MintEmitsZeroHashEventsWhenRecipientHasNoKey() public {
        vm.recordLogs();
        token.mint(recipient, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], address(0), recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], address(0), recipient, bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    BURN: SENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_BurnEmitsEncryptedEventToSenderWhenSenderHasKey() public {
        _registerKey(sender);

        bytes32 senderKeyHash = directory.keyHash(sender);

        token.mint(sender, suint256(1e18));

        vm.recordLogs();
        token.burn(sender, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        // sender has key → encrypted
        _assertTransferLog(transferLogs[0], sender, address(0), senderKeyHash);
        _assertEncryptedDataNonEmpty(transferLogs[0]);
        // address(0) has no key → zero hash
        _assertTransferLog(transferLogs[1], sender, address(0), bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    BURN: SENDER HAS NO KEY
    //////////////////////////////////////////////////////////////*/

    function test_BurnEmitsZeroHashEventsWhenSenderHasNoKey() public {
        token.mint(sender, suint256(1e18));

        vm.recordLogs();
        token.burn(sender, suint256(1e18));

        Vm.Log[] memory transferLogs = _getTransferLogs();
        assertEq(transferLogs.length, 2);
        _assertTransferLog(transferLogs[0], sender, address(0), bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[0]);
        _assertTransferLog(transferLogs[1], sender, address(0), bytes32(0));
        _assertEncryptedDataEmpty(transferLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                            HELPERS
    //////////////////////////////////////////////////////////////*/

    function _registerKey(address user) internal {
        suint256 key = CryptoUtils.generateRandomAESKey();
        vm.prank(user);
        directory.setKey(key);
    }

    function _getTransferLogs() internal returns (Vm.Log[] memory) {
        Vm.Log[] memory allLogs = vm.getRecordedLogs();

        // Count Transfer events
        uint256 count = 0;
        for (uint256 i = 0; i < allLogs.length; i++) {
            if (allLogs[i].topics[0] == TRANSFER_TOPIC) {
                count++;
            }
        }

        // Collect Transfer events
        Vm.Log[] memory transferLogs = new Vm.Log[](count);
        uint256 idx = 0;
        for (uint256 i = 0; i < allLogs.length; i++) {
            if (allLogs[i].topics[0] == TRANSFER_TOPIC) {
                transferLogs[idx++] = allLogs[i];
            }
        }

        return transferLogs;
    }

    function _assertTransferLog(Vm.Log memory log, address from, address to, bytes32 encryptKeyHash) internal pure {
        assertEq(log.topics[1], bytes32(uint256(uint160(from))));
        assertEq(log.topics[2], bytes32(uint256(uint160(to))));
        assertEq(log.topics[3], encryptKeyHash);
    }

    function _assertEncryptedDataNonEmpty(Vm.Log memory log) internal pure {
        bytes memory encryptedAmount = abi.decode(log.data, (bytes));
        assertTrue(encryptedAmount.length > 0, "expected non-empty encrypted data");
    }

    function _assertEncryptedDataEmpty(Vm.Log memory log) internal pure {
        bytes memory encryptedAmount = abi.decode(log.data, (bytes));
        assertEq(encryptedAmount.length, 0, "expected empty encrypted data");
    }
}
