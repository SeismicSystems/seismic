// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ProtocolParams} from "../src/seismic-std-lib/ProtocolParams.sol";

contract ProtocolParamsTest is Test {
    ProtocolParams public protocolParams;

    address public owner;
    address public alice;
    address public bob;

    event ProtocolParamEvent(uint8 param_id, bytes param);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    function setUp() public {
        owner = address(this);
        alice = makeAddr("alice");
        bob = makeAddr("bob");

        protocolParams = new ProtocolParams();
    }

    // ============ Constructor Tests ============

    function test_ConstructorSetsOwner() public view {
        assertEq(protocolParams.owner(), owner);
    }

    function test_ConstructorEmitsOwnershipTransferred() public {
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));
        new ProtocolParams();
    }

    function test_MaxParamLengthIsCorrect() public view {
        assertEq(protocolParams.MAX_PARAM_LENGTH(), 100);
    }

    // ============ set_param Tests ============

    function test_SetParam() public {
        bytes memory value = hex"01020304";

        protocolParams.set_param(1, value);

        assertEq(protocolParams.get_param(1), value);
    }

    function test_SetParamEmitsEvent() public {
        bytes memory value = hex"01020304";

        vm.expectEmit(true, true, false, true);
        emit ProtocolParamEvent(1, value);

        protocolParams.set_param(1, value);
    }

    function test_SetParamOverwritesExisting() public {
        bytes memory value1 = hex"01020304";
        bytes memory value2 = hex"cafebabe";

        protocolParams.set_param(1, value1);
        protocolParams.set_param(1, value2);

        assertEq(protocolParams.get_param(1), value2);
    }

    function test_SetParamWithEmptyBytes() public {
        bytes memory emptyValue = "";

        protocolParams.set_param(1, emptyValue);

        assertEq(protocolParams.get_param(1), emptyValue);
        assertFalse(protocolParams.has_param(1));
    }

    function test_SetParamAtMaxLength() public {
        bytes memory maxValue = new bytes(100);
        for (uint i = 0; i < 100; i++) {
            maxValue[i] = bytes1(uint8(i));
        }

        protocolParams.set_param(1, maxValue);

        assertEq(protocolParams.get_param(1), maxValue);
    }

    function test_RevertWhen_SetParamExceedsMaxLength() public {
        bytes memory tooLong = new bytes(101);

        vm.expectRevert(
            abi.encodeWithSelector(ProtocolParams.ParamTooLarge.selector, 101, 100)
        );
        protocolParams.set_param(1, tooLong);
    }

    function test_RevertWhen_SetParamCalledByNonOwner() public {
        bytes memory value = hex"01020304";

        vm.prank(alice);
        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.set_param(1, value);
    }

    // ============ get_param Tests ============

    function test_GetParamReturnsEmptyForUnsetParam() public view {
        bytes memory result = protocolParams.get_param(1);
        assertEq(result.length, 0);
    }

    function test_GetParamReturnsCorrectValue() public {
        bytes memory value = hex"0102030405";
        protocolParams.set_param(42, value);

        assertEq(protocolParams.get_param(42), value);
    }

    function test_GetParamDifferentIds() public {
        bytes memory value1 = hex"11";
        bytes memory value2 = hex"22";
        bytes memory value3 = hex"33";

        protocolParams.set_param(0, value1);
        protocolParams.set_param(128, value2);
        protocolParams.set_param(255, value3);

        assertEq(protocolParams.get_param(0), value1);
        assertEq(protocolParams.get_param(128), value2);
        assertEq(protocolParams.get_param(255), value3);
    }

    // ============ has_param Tests ============

    function test_HasParamReturnsFalseForUnsetParam() public view {
        assertFalse(protocolParams.has_param(1));
    }

    function test_HasParamReturnsTrueForSetParam() public {
        protocolParams.set_param(1, hex"01020304");
        assertTrue(protocolParams.has_param(1));
    }

    function test_HasParamReturnsFalseAfterSettingEmptyBytes() public {
        protocolParams.set_param(1, hex"01020304");
        assertTrue(protocolParams.has_param(1));

        protocolParams.set_param(1, "");
        assertFalse(protocolParams.has_param(1));
    }

    // ============ transferOwnership Tests ============

    function test_TransferOwnership() public {
        protocolParams.transferOwnership(alice);
        assertEq(protocolParams.owner(), alice);
    }

    function test_TransferOwnershipEmitsEvent() public {
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(owner, alice);

        protocolParams.transferOwnership(alice);
    }

    function test_TransferOwnershipAllowsNewOwnerToSetParam() public {
        protocolParams.transferOwnership(alice);

        vm.prank(alice);
        protocolParams.set_param(1, hex"01020304");

        assertEq(protocolParams.get_param(1), hex"01020304");
    }

    function test_TransferOwnershipRevokesOldOwnerAccess() public {
        protocolParams.transferOwnership(alice);

        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.set_param(1, hex"01020304");
    }

    function test_RevertWhen_TransferOwnershipToZeroAddress() public {
        vm.expectRevert(ProtocolParams.ZeroAddress.selector);
        protocolParams.transferOwnership(address(0));
    }

    function test_RevertWhen_TransferOwnershipCalledByNonOwner() public {
        vm.prank(alice);
        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.transferOwnership(bob);
    }

    // ============ renounceOwnership Tests ============

    function test_RenounceOwnership() public {
        protocolParams.renounceOwnership();
        assertEq(protocolParams.owner(), address(0));
    }

    function test_RenounceOwnershipEmitsEvent() public {
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(owner, address(0));

        protocolParams.renounceOwnership();
    }

    function test_RevertWhen_SetParamAfterRenounceOwnership() public {
        protocolParams.renounceOwnership();

        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.set_param(1, hex"01020304");
    }

    function test_RevertWhen_TransferOwnershipAfterRenounce() public {
        protocolParams.renounceOwnership();

        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.transferOwnership(alice);
    }

    function test_RevertWhen_RenounceOwnershipCalledByNonOwner() public {
        vm.prank(alice);
        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.renounceOwnership();
    }

    // ============ Fuzz Tests ============

    function testFuzz_SetAndGetParam(uint8 paramId, bytes calldata value) public {
        vm.assume(value.length <= 100);

        protocolParams.set_param(paramId, value);

        assertEq(protocolParams.get_param(paramId), value);
        if (value.length > 0) {
            assertTrue(protocolParams.has_param(paramId));
        } else {
            assertFalse(protocolParams.has_param(paramId));
        }
    }

    function testFuzz_RevertWhen_ParamTooLarge(uint8 paramId, uint256 length) public {
        vm.assume(length > 100);
        vm.assume(length <= 1000); // reasonable upper bound

        bytes memory tooLong = new bytes(length);

        vm.expectRevert(
            abi.encodeWithSelector(ProtocolParams.ParamTooLarge.selector, length, 100)
        );
        protocolParams.set_param(paramId, tooLong);
    }

    function testFuzz_RevertWhen_NonOwnerSetsParam(address caller, uint8 paramId, bytes calldata value) public {
        vm.assume(caller != owner);
        vm.assume(value.length <= 100);

        vm.prank(caller);
        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.set_param(paramId, value);
    }

    function testFuzz_TransferOwnership(address newOwner) public {
        vm.assume(newOwner != address(0));

        protocolParams.transferOwnership(newOwner);

        assertEq(protocolParams.owner(), newOwner);
    }

    // ============ Edge Cases ============

    function test_SetParamWithSingleByte() public {
        bytes memory singleByte = hex"ff";

        protocolParams.set_param(0, singleByte);

        assertEq(protocolParams.get_param(0), singleByte);
        assertTrue(protocolParams.has_param(0));
    }

    function test_SetParamAtMinAndMaxParamId() public {
        bytes memory value = hex"01020304";

        protocolParams.set_param(0, value);
        protocolParams.set_param(255, value);

        assertEq(protocolParams.get_param(0), value);
        assertEq(protocolParams.get_param(255), value);
    }

    function test_MultipleOwnersInSequence() public {
        // Owner -> Alice
        protocolParams.transferOwnership(alice);

        // Alice -> Bob
        vm.prank(alice);
        protocolParams.transferOwnership(bob);

        // Bob sets param
        vm.prank(bob);
        protocolParams.set_param(1, hex"b0b5");

        assertEq(protocolParams.owner(), bob);
        assertEq(protocolParams.get_param(1), hex"b0b5");

        // Old owners can't set params
        vm.prank(alice);
        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.set_param(2, hex"fa11");

        vm.expectRevert(ProtocolParams.OnlyOwner.selector);
        protocolParams.set_param(2, hex"fa11");
    }
}
