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
    bytes32 constant APPROVAL_TOPIC = keccak256("Approval(address,address,bytes32,bytes)");
    bytes32 constant PERMIT_TYPEHASH =
        keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

    address owner = makeAddr("owner");
    address spender = makeAddr("spender");

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
        _assertDecryptsTo(transferLogs[0], sender, 1e18);
        _assertDecryptsTo(transferLogs[1], recipient, 1e18);
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
        _assertDecryptsTo(transferLogs[0], sender, 1e18);
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
        _assertDecryptsTo(transferLogs[1], recipient, 1e18);
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
        _assertDecryptsTo(transferLogs[0], sender, 1e18);
        _assertDecryptsTo(transferLogs[1], recipient, 1e18);
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
        _assertDecryptsTo(transferLogs[0], sender, 1e18);
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
        _assertDecryptsTo(transferLogs[1], recipient, 1e18);
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
        // recipient has key → encrypted, decryptable
        _assertTransferLog(transferLogs[1], address(0), recipient, recipientKeyHash);
        _assertDecryptsTo(transferLogs[1], recipient, 1e18);
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
        // sender has key → encrypted, decryptable
        _assertTransferLog(transferLogs[0], sender, address(0), senderKeyHash);
        _assertDecryptsTo(transferLogs[0], sender, 1e18);
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
                    APPROVE: BOTH PARTIES HAVE KEYS
    //////////////////////////////////////////////////////////////*/

    function test_ApproveEmitsEventToBothPartiesWhenBothHaveKeys() public {
        _registerKey(owner);
        _registerKey(spender);

        bytes32 ownerKeyHash = directory.keyHash(owner);
        bytes32 spenderKeyHash = directory.keyHash(spender);

        vm.recordLogs();
        vm.prank(owner);
        token.approve(spender, suint256(1e18));

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], owner, spender, ownerKeyHash);
        _assertApprovalLog(approvalLogs[1], owner, spender, spenderKeyHash);
        _assertDecryptsTo(approvalLogs[0], owner, 1e18);
        _assertDecryptsTo(approvalLogs[1], spender, 1e18);
    }

    /*//////////////////////////////////////////////////////////////
                    APPROVE: ONLY OWNER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_ApproveEmitsEncryptedEventToOwnerWhenOnlyOwnerHasKey() public {
        _registerKey(owner);

        bytes32 ownerKeyHash = directory.keyHash(owner);

        vm.recordLogs();
        vm.prank(owner);
        token.approve(spender, suint256(1e18));

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], owner, spender, ownerKeyHash);
        _assertDecryptsTo(approvalLogs[0], owner, 1e18);
        _assertApprovalLog(approvalLogs[1], owner, spender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    APPROVE: ONLY SPENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_ApproveEmitsEncryptedEventToSpenderWhenOnlySpenderHasKey() public {
        _registerKey(spender);

        bytes32 spenderKeyHash = directory.keyHash(spender);

        vm.recordLogs();
        vm.prank(owner);
        token.approve(spender, suint256(1e18));

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], owner, spender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[0]);
        _assertApprovalLog(approvalLogs[1], owner, spender, spenderKeyHash);
        _assertDecryptsTo(approvalLogs[1], spender, 1e18);
    }

    /*//////////////////////////////////////////////////////////////
                    APPROVE: NEITHER PARTY HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_ApproveEmitsZeroHashEventsToBothPartiesWhenNeitherHasKey() public {
        vm.recordLogs();
        vm.prank(owner);
        token.approve(spender, suint256(1e18));

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], owner, spender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[0]);
        _assertApprovalLog(approvalLogs[1], owner, spender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    PERMIT: BOTH PARTIES HAVE KEYS
    //////////////////////////////////////////////////////////////*/

    function test_PermitEmitsEventToBothPartiesWhenBothHaveKeys() public {
        (address permitOwner, uint256 privateKey) = makeAddrAndKey("permitOwner");
        address permitSpender = makeAddr("permitSpender");

        _registerKey(permitOwner);
        _registerKey(permitSpender);

        bytes32 ownerKeyHash = directory.keyHash(permitOwner);
        bytes32 spenderKeyHash = directory.keyHash(permitSpender);

        (uint8 v, bytes32 r, bytes32 s) = _signPermit(privateKey, permitOwner, permitSpender, 1e18, 0, block.timestamp);

        vm.recordLogs();
        token.permit(permitOwner, permitSpender, suint256(1e18), block.timestamp, v, r, s);

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], permitOwner, permitSpender, ownerKeyHash);
        _assertApprovalLog(approvalLogs[1], permitOwner, permitSpender, spenderKeyHash);
        _assertDecryptsTo(approvalLogs[0], permitOwner, 1e18);
        _assertDecryptsTo(approvalLogs[1], permitSpender, 1e18);
    }

    /*//////////////////////////////////////////////////////////////
                    PERMIT: ONLY OWNER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_PermitEmitsEncryptedEventToOwnerWhenOnlyOwnerHasKey() public {
        (address permitOwner, uint256 privateKey) = makeAddrAndKey("permitOwner");
        address permitSpender = makeAddr("permitSpender");

        _registerKey(permitOwner);

        bytes32 ownerKeyHash = directory.keyHash(permitOwner);

        (uint8 v, bytes32 r, bytes32 s) = _signPermit(privateKey, permitOwner, permitSpender, 1e18, 0, block.timestamp);

        vm.recordLogs();
        token.permit(permitOwner, permitSpender, suint256(1e18), block.timestamp, v, r, s);

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], permitOwner, permitSpender, ownerKeyHash);
        _assertDecryptsTo(approvalLogs[0], permitOwner, 1e18);
        _assertApprovalLog(approvalLogs[1], permitOwner, permitSpender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[1]);
    }

    /*//////////////////////////////////////////////////////////////
                    PERMIT: ONLY SPENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_PermitEmitsEncryptedEventToSpenderWhenOnlySpenderHasKey() public {
        (address permitOwner, uint256 privateKey) = makeAddrAndKey("permitOwner");
        address permitSpender = makeAddr("permitSpender");

        _registerKey(permitSpender);

        bytes32 spenderKeyHash = directory.keyHash(permitSpender);

        (uint8 v, bytes32 r, bytes32 s) = _signPermit(privateKey, permitOwner, permitSpender, 1e18, 0, block.timestamp);

        vm.recordLogs();
        token.permit(permitOwner, permitSpender, suint256(1e18), block.timestamp, v, r, s);

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], permitOwner, permitSpender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[0]);
        _assertApprovalLog(approvalLogs[1], permitOwner, permitSpender, spenderKeyHash);
        _assertDecryptsTo(approvalLogs[1], permitSpender, 1e18);
    }

    /*//////////////////////////////////////////////////////////////
                    PERMIT: NEITHER PARTY HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_PermitEmitsZeroHashEventsToBothPartiesWhenNeitherHasKey() public {
        (address permitOwner, uint256 privateKey) = makeAddrAndKey("permitOwner");
        address permitSpender = makeAddr("permitSpender");

        (uint8 v, bytes32 r, bytes32 s) = _signPermit(privateKey, permitOwner, permitSpender, 1e18, 0, block.timestamp);

        vm.recordLogs();
        token.permit(permitOwner, permitSpender, suint256(1e18), block.timestamp, v, r, s);

        Vm.Log[] memory approvalLogs = _getApprovalLogs();
        assertEq(approvalLogs.length, 2);
        _assertApprovalLog(approvalLogs[0], permitOwner, permitSpender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[0]);
        _assertApprovalLog(approvalLogs[1], permitOwner, permitSpender, bytes32(0));
        _assertEncryptedDataEmpty(approvalLogs[1]);
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

    function _getApprovalLogs() internal returns (Vm.Log[] memory) {
        Vm.Log[] memory allLogs = vm.getRecordedLogs();

        // Count Approval events
        uint256 count = 0;
        for (uint256 i = 0; i < allLogs.length; i++) {
            if (allLogs[i].topics[0] == APPROVAL_TOPIC) {
                count++;
            }
        }

        // Collect Approval events
        Vm.Log[] memory approvalLogs = new Vm.Log[](count);
        uint256 idx = 0;
        for (uint256 i = 0; i < allLogs.length; i++) {
            if (allLogs[i].topics[0] == APPROVAL_TOPIC) {
                approvalLogs[idx++] = allLogs[i];
            }
        }

        return approvalLogs;
    }

    function _assertApprovalLog(Vm.Log memory log, address _owner, address _spender, bytes32 encryptKeyHash)
        internal
        pure
    {
        assertEq(log.topics[1], bytes32(uint256(uint160(_owner))));
        assertEq(log.topics[2], bytes32(uint256(uint160(_spender))));
        assertEq(log.topics[3], encryptKeyHash);
    }

    function _assertTransferLog(Vm.Log memory log, address from, address to, bytes32 encryptKeyHash) internal pure {
        assertEq(log.topics[1], bytes32(uint256(uint160(from))));
        assertEq(log.topics[2], bytes32(uint256(uint160(to))));
        assertEq(log.topics[3], encryptKeyHash);
    }

    function _assertDecryptsTo(Vm.Log memory log, address keyOwner, uint256 expectedAmount) internal {
        bytes memory encryptedAmount = abi.decode(log.data, (bytes));
        assertTrue(encryptedAmount.length > 0, "expected non-empty encrypted data");

        vm.prank(keyOwner);
        bytes memory decrypted = directory.decrypt(encryptedAmount);
        uint256 amount = abi.decode(decrypted, (uint256));
        assertEq(amount, expectedAmount, "decrypted amount does not match");
    }

    function _assertEncryptedDataEmpty(Vm.Log memory log) internal pure {
        bytes memory encryptedAmount = abi.decode(log.data, (bytes));
        assertEq(encryptedAmount.length, 0, "expected empty encrypted data");
    }

    function _signPermit(
        uint256 privateKey,
        address _owner,
        address _spender,
        uint256 value,
        uint256 nonce,
        uint256 deadline
    ) internal view returns (uint8 v, bytes32 r, bytes32 s) {
        (v, r, s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, _owner, _spender, value, nonce, deadline))
                )
            )
        );
    }
}
