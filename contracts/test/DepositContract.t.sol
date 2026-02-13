// SPDX-License-Identifier: CC0-1.0
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";
import {DepositContract, IDepositContract, ERC165} from "../src/seismic-std-lib/DepositContract.sol";

contract DepositContractTest is Test {
    DepositContract public depositContract;

    // Test data - valid lengths
    bytes constant NODE_PUBKEY = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 32 bytes
    bytes constant CONSENSUS_PUBKEY = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 48 bytes
    bytes constant WITHDRAWAL_CREDENTIALS = hex"0100000000000000000000001234567890abcdef1234567890abcdef12345678"; // 32 bytes
    bytes constant NODE_SIGNATURE = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 64 bytes
    bytes constant CONSENSUS_SIGNATURE = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 96 bytes

    event DepositEvent(
        bytes node_pubkey,
        bytes consensus_pubkey,
        bytes withdrawal_credentials,
        bytes amount,
        bytes node_signature,
        bytes consensus_signature,
        bytes index
    );

    function setUp() public {
        depositContract = new DepositContract();
    }

    // Helper function to compute deposit data root (mirrors contract logic)
    function computeDepositDataRoot(
        bytes memory node_pubkey,
        bytes memory consensus_pubkey,
        bytes memory withdrawal_credentials,
        bytes memory node_signature,
        bytes memory consensus_signature,
        uint64 amount
    ) internal pure returns (bytes32) {
        bytes memory amountBytes = to_little_endian_64(amount);

        bytes32 consensus_pubkey_hash = sha256(abi.encodePacked(consensus_pubkey, bytes16(0)));
        bytes32 pubkey_root = sha256(abi.encodePacked(node_pubkey, consensus_pubkey_hash));
        bytes32 node_signature_hash = sha256(node_signature);

        bytes memory consensus_sig_first = new bytes(64);
        bytes memory consensus_sig_second = new bytes(32);
        for (uint i = 0; i < 64; i++) {
            consensus_sig_first[i] = consensus_signature[i];
        }
        for (uint i = 0; i < 32; i++) {
            consensus_sig_second[i] = consensus_signature[64 + i];
        }

        bytes32 consensus_signature_hash = sha256(abi.encodePacked(
            sha256(consensus_sig_first),
            sha256(abi.encodePacked(consensus_sig_second, bytes32(0)))
        ));
        bytes32 signature_root = sha256(abi.encodePacked(node_signature_hash, consensus_signature_hash));

        return sha256(abi.encodePacked(
            sha256(abi.encodePacked(pubkey_root, withdrawal_credentials)),
            sha256(abi.encodePacked(amountBytes, bytes24(0), signature_root))
        ));
    }

    function to_little_endian_64(uint64 value) internal pure returns (bytes memory ret) {
        ret = new bytes(8);
        bytes8 bytesValue = bytes8(value);
        ret[0] = bytesValue[7];
        ret[1] = bytesValue[6];
        ret[2] = bytesValue[5];
        ret[3] = bytesValue[4];
        ret[4] = bytesValue[3];
        ret[5] = bytesValue[2];
        ret[6] = bytesValue[1];
        ret[7] = bytesValue[0];
    }

    // ============ Success Cases ============

    function test_SuccessfulDeposit() public {
        uint256 depositAmount = 32 ether;
        uint64 amountInGwei = uint64(depositAmount / 1 gwei);

        bytes32 depositDataRoot = computeDepositDataRoot(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            amountInGwei
        );

        vm.expectEmit(true, true, true, true);
        emit DepositEvent(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            to_little_endian_64(amountInGwei),
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            to_little_endian_64(0) // first deposit, index 0
        );

        depositContract.deposit{value: depositAmount}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            depositDataRoot
        );

        // Verify deposit count
        bytes memory countBytes = depositContract.get_deposit_count();
        assertEq(countBytes.length, 8);
        assertEq(countBytes[0], bytes1(0x01)); // little endian 1
    }

    function test_MinimumDeposit() public {
        uint256 depositAmount = 1 ether;
        uint64 amountInGwei = uint64(depositAmount / 1 gwei);

        bytes32 depositDataRoot = computeDepositDataRoot(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            amountInGwei
        );

        depositContract.deposit{value: depositAmount}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            depositDataRoot
        );

        bytes memory countBytes = depositContract.get_deposit_count();
        assertEq(countBytes[0], bytes1(0x01));
    }

    function test_MultipleDeposits() public {
        uint256 depositAmount = 32 ether;
        uint64 amountInGwei = uint64(depositAmount / 1 gwei);

        bytes32 depositDataRoot = computeDepositDataRoot(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            amountInGwei
        );

        // First deposit
        depositContract.deposit{value: depositAmount}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            depositDataRoot
        );

        // Second deposit (same data is allowed)
        depositContract.deposit{value: depositAmount}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            depositDataRoot
        );

        // Verify deposit count is 2
        bytes memory countBytes = depositContract.get_deposit_count();
        assertEq(countBytes[0], bytes1(0x02)); // little endian 2
    }

    // ============ Invalid Input Length Cases ============

    function test_RevertWhen_NodePubkeyTooShort() public {
        bytes memory shortNodePubkey = hex"1234567890abcdef1234567890abcdef"; // 16 bytes instead of 32

        vm.expectRevert("DepositContract: invalid node_pubkey length");
        depositContract.deposit{value: 32 ether}(
            shortNodePubkey,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_NodePubkeyTooLong() public {
        bytes memory longNodePubkey = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 40 bytes

        vm.expectRevert("DepositContract: invalid node_pubkey length");
        depositContract.deposit{value: 32 ether}(
            longNodePubkey,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_ConsensusPubkeyTooShort() public {
        bytes memory shortConsensusPubkey = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 32 bytes instead of 48

        vm.expectRevert("DepositContract: invalid consensus_pubkey length");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            shortConsensusPubkey,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_ConsensusPubkeyTooLong() public {
        bytes memory longConsensusPubkey = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 64 bytes

        vm.expectRevert("DepositContract: invalid consensus_pubkey length");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            longConsensusPubkey,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_WithdrawalCredentialsTooShort() public {
        bytes memory shortCredentials = hex"0100000000000000000000001234567890abcdef"; // 20 bytes instead of 32

        vm.expectRevert("DepositContract: invalid withdrawal_credentials length");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            shortCredentials,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_WithdrawalCredentialsInvalidPrefix() public {
        // Valid length but wrong prefix (0x00 instead of 0x01)
        bytes memory invalidPrefixCredentials = hex"0000000000000000000000001234567890abcdef1234567890abcdef12345678";

        vm.expectRevert("DepositContract: invalid withdrawal_credentials prefix");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            invalidPrefixCredentials,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_WithdrawalCredentialsInvalidPrefix02() public {
        // Valid length but wrong prefix (0x02)
        bytes memory invalidPrefixCredentials = hex"0200000000000000000000001234567890abcdef1234567890abcdef12345678";

        vm.expectRevert("DepositContract: invalid withdrawal_credentials prefix");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            invalidPrefixCredentials,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_WithdrawalCredentialsInvalidPadding() public {
        // Valid prefix but non-zero padding bytes (byte 1 is 0xFF)
        bytes memory invalidPaddingCredentials = hex"01FF000000000000000000001234567890abcdef1234567890abcdef12345678";

        vm.expectRevert("DepositContract: invalid withdrawal_credentials padding");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            invalidPaddingCredentials,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_WithdrawalCredentialsInvalidPaddingAtEnd() public {
        // Valid prefix but non-zero byte at position 11 (last padding byte)
        bytes memory invalidPaddingCredentials = hex"0100000000000000000000011234567890abcdef1234567890abcdef12345678";

        vm.expectRevert("DepositContract: invalid withdrawal_credentials padding");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            invalidPaddingCredentials,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_NodeSignatureTooShort() public {
        bytes memory shortNodeSig = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 32 bytes instead of 64

        vm.expectRevert("DepositContract: invalid node_signature length");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            shortNodeSig,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_ConsensusSignatureTooShort() public {
        bytes memory shortConsensusSig = hex"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"; // 64 bytes instead of 96

        vm.expectRevert("DepositContract: invalid consensus_signature length");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            shortConsensusSig,
            bytes32(0)
        );
    }

    // ============ Invalid Deposit Amount Cases ============

    function test_RevertWhen_DepositTooLow() public {
        vm.expectRevert("DepositContract: deposit value too low");
        depositContract.deposit{value: 0.5 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_DepositZero() public {
        vm.expectRevert("DepositContract: deposit value too low");
        depositContract.deposit{value: 0}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function test_RevertWhen_DepositNotGweiMultiple() public {
        vm.expectRevert("DepositContract: deposit value not multiple of gwei");
        depositContract.deposit{value: 1 ether + 1 wei}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    // ============ Invalid Deposit Data Root Cases ============

    function test_RevertWhen_WrongDepositDataRoot() public {
        bytes32 wrongRoot = bytes32(uint256(1)); // intentionally wrong

        vm.expectRevert("DepositContract: reconstructed DepositData does not match supplied deposit_data_root");
        depositContract.deposit{value: 32 ether}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            wrongRoot
        );
    }

    // ============ ERC165 Interface Support ============

    function test_SupportsERC165Interface() public view {
        bytes4 erc165InterfaceId = 0x01ffc9a7; // type(ERC165).interfaceId
        assertTrue(depositContract.supportsInterface(erc165InterfaceId));
    }

    function test_SupportsDepositContractInterface() public view {
        bytes4 depositContractInterfaceId = type(IDepositContract).interfaceId;
        assertTrue(depositContract.supportsInterface(depositContractInterfaceId));
    }

    function test_DoesNotSupportInvalidInterface() public view {
        bytes4 invalidInterfaceId = 0xffffffff;
        assertFalse(depositContract.supportsInterface(invalidInterfaceId));
    }

    function test_DoesNotSupportRandomInterface() public view {
        bytes4 randomInterfaceId = 0x12345678;
        assertFalse(depositContract.supportsInterface(randomInterfaceId));
    }

    // ============ Deposit Root and Count ============

    function test_InitialDepositCount() public view {
        bytes memory countBytes = depositContract.get_deposit_count();
        assertEq(countBytes.length, 8);
        // All bytes should be 0 for count of 0
        for (uint i = 0; i < 8; i++) {
            assertEq(countBytes[i], bytes1(0x00));
        }
    }

    function test_DepositRootChangesAfterDeposit() public {
        bytes32 initialRoot = depositContract.get_deposit_root();

        uint256 depositAmount = 32 ether;
        uint64 amountInGwei = uint64(depositAmount / 1 gwei);

        bytes32 depositDataRoot = computeDepositDataRoot(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            amountInGwei
        );

        depositContract.deposit{value: depositAmount}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            depositDataRoot
        );

        bytes32 newRoot = depositContract.get_deposit_root();
        assertTrue(initialRoot != newRoot, "Deposit root should change after deposit");
    }

    // ============ Fuzz Tests ============

    function testFuzz_RevertWhen_DepositBelowMinimum(uint256 amount) public {
        vm.assume(amount < 1 ether);
        vm.assume(amount % 1 gwei == 0); // valid gwei multiple but below minimum

        vm.expectRevert("DepositContract: deposit value too low");
        depositContract.deposit{value: amount}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            bytes32(0)
        );
    }

    function testFuzz_SuccessfulDepositWithValidAmount(uint64 gweiAmount) public {
        vm.assume(gweiAmount >= 1_000_000_000); // at least 1 ether in gwei
        vm.assume(gweiAmount <= 100_000_000_000); // max 100 ether to keep test reasonable

        uint256 depositAmount = uint256(gweiAmount) * 1 gwei;

        bytes32 depositDataRoot = computeDepositDataRoot(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            gweiAmount
        );

        vm.deal(address(this), depositAmount);
        depositContract.deposit{value: depositAmount}(
            NODE_PUBKEY,
            CONSENSUS_PUBKEY,
            WITHDRAWAL_CREDENTIALS,
            NODE_SIGNATURE,
            CONSENSUS_SIGNATURE,
            depositDataRoot
        );

        bytes memory countBytes = depositContract.get_deposit_count();
        assertEq(countBytes[0], bytes1(0x01));
    }
}
