// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {MeasurementRegistry} from "../src/enclave/MeasurementRegistry.sol";
import {MeasurementAuthorityDev} from "../src/enclave/MeasurementAuthorityDev.sol";

contract MeasurementAuthorityDevTest is Test {
    bytes32 internal constant REGISTRY_STORAGE_LOCATION =
        0xa3ae60943e4f183142036d77b94858085814dd428f131289aea7e42703fb0b00;

    address internal constant REGISTRY_ADDRESS = 0x1000000000000000000000000000000000000001;
    address internal constant AUTHORITY_ADDRESS = 0x1000000000000000000000000000000000000002;

    address internal constant OWNER = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    bytes32 internal constant BOOTSTRAP_POLICY_HASH = keccak256("bootstrap-policy");
    bytes32 internal constant INITIAL_ADMISSION_ID = bytes32(uint256(1));

    MeasurementRegistry internal registry;
    MeasurementAuthorityDev internal authority;

    function setUp() public {
        // Both contracts are genesis predeploys with empty constructors, so tests
        // etch their runtime code at the fixed addresses the contracts hardcode
        // for each other and write the registry's initial state directly.
        vm.etch(REGISTRY_ADDRESS, address(new MeasurementRegistry()).code);
        registry = MeasurementRegistry(REGISTRY_ADDRESS);
        _initializeRegistry();

        vm.etch(AUTHORITY_ADDRESS, address(new MeasurementAuthorityDev()).code);
        authority = MeasurementAuthorityDev(AUTHORITY_ADDRESS);
    }

    function test_WellKnownAddressesMatch() public view {
        assertEq(registry.AUTHORITY(), address(authority));
        assertEq(address(authority.REGISTRY()), address(registry));
    }

    function test_OwnerAcceptsNewAdmissionId() public {
        bytes32 admissionId = keccak256("new-admission");
        bytes32 newPolicyHash = keccak256("policy-2");

        vm.prank(OWNER);
        authority.applyPolicyUpdate(_singleton(admissionId), _empty(), newPolicyHash);

        assertTrue(registry.isAccepted(admissionId));
        assertEq(registry.activePolicyHash(), newPolicyHash);
        assertEq(registry.policyRevision(), 2);
        assertEq(registry.acceptedCount(), 2);
    }

    function test_OwnerDeprecatesAcceptedAdmissionId() public {
        vm.prank(OWNER);
        authority.applyPolicyUpdate(_empty(), _singleton(INITIAL_ADMISSION_ID), keccak256("policy-2"));

        assertFalse(registry.isAccepted(INITIAL_ADMISSION_ID));
        assertEq(registry.acceptedCount(), 0);
    }

    function test_RevertWhen_CallerIsNotOwner() public {
        address caller = makeAddr("caller");

        vm.prank(caller);
        vm.expectRevert(abi.encodeWithSelector(MeasurementAuthorityDev.NotOwner.selector, caller));
        authority.applyPolicyUpdate(_singleton(keccak256("id")), _empty(), keccak256("policy-2"));
    }

    function test_RegistryRejectionBubblesUp() public {
        vm.prank(OWNER);
        vm.expectRevert(
            abi.encodeWithSelector(
                MeasurementRegistry.InvalidTransition.selector,
                INITIAL_ADMISSION_ID,
                MeasurementRegistry.Status.Accepted,
                MeasurementRegistry.Status.Accepted
            )
        );
        authority.applyPolicyUpdate(_singleton(INITIAL_ADMISSION_ID), _empty(), keccak256("policy-2"));

        assertEq(registry.policyRevision(), 1);
    }

    function _initializeRegistry() internal {
        vm.store(REGISTRY_ADDRESS, bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 1), BOOTSTRAP_POLICY_HASH);
        vm.store(REGISTRY_ADDRESS, bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 2), BOOTSTRAP_POLICY_HASH);
        vm.store(REGISTRY_ADDRESS, bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 3), bytes32(uint256(1)));
        vm.store(REGISTRY_ADDRESS, bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 4), bytes32(uint256(1)));
        vm.store(
            REGISTRY_ADDRESS,
            keccak256(abi.encode(INITIAL_ADMISSION_ID, REGISTRY_STORAGE_LOCATION)),
            bytes32(uint256(uint8(MeasurementRegistry.Status.Accepted)))
        );
    }

    function _singleton(bytes32 value) internal pure returns (bytes32[] memory values) {
        values = new bytes32[](1);
        values[0] = value;
    }

    function _empty() internal pure returns (bytes32[] memory values) {
        values = new bytes32[](0);
    }
}
