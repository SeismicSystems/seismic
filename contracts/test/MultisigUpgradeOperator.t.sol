// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import {MultisigUpgradeOperator} from "../src/enclave/MultisigUpgradeOperator.sol";
import {UpgradeOperator} from "../src/enclave/UpgradeOperator.sol";

contract MultisigUpgradeOperatorTest is Test {
    address internal constant SIGNER1 = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    address internal constant UPGRADE_OPERATOR = 0x1000000000000000000000000000000000000001;

    MultisigUpgradeOperator internal multisig;

    function setUp() public {
        UpgradeOperator implementation = new UpgradeOperator();
        vm.etch(UPGRADE_OPERATOR, address(implementation).code);
        multisig = new MultisigUpgradeOperator();
    }

    function _createProposal() internal returns (bytes32) {
        return multisig.createProposalV1(new bytes(48), new bytes(48), new bytes(32), true);
    }

    function test_RevertWhen_SignerVotesTwiceAfterReject() public {
        bytes32 proposalId = _createProposal();

        vm.prank(SIGNER1);
        multisig.vote(proposalId, false);

        vm.prank(SIGNER1);
        vm.expectRevert("Already voted");
        multisig.vote(proposalId, true);
    }

    function test_GetVoteCountIncludesRejectedVotes() public {
        bytes32 proposalId = _createProposal();

        vm.prank(SIGNER1);
        multisig.vote(proposalId, false);

        (uint256 approvalCount, uint256 totalVotes) = multisig.getVoteCount(proposalId);
        assertEq(approvalCount, 0);
        assertEq(totalVotes, 1);
    }
}