// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {MeasurementRegistry} from "../src/enclave/MeasurementRegistry.sol";

contract MeasurementRegistryTest is Test {
    bytes32 internal constant REGISTRY_STORAGE_LOCATION =
        0xa3ae60943e4f183142036d77b94858085814dd428f131289aea7e42703fb0b00;

    address internal constant AUTHORITY = 0x1000000000000000000000000000000000000002;

    bytes32 internal constant BOOTSTRAP_POLICY_HASH = keccak256("bootstrap-policy");
    bytes32 internal constant INITIAL_ADMISSION_ID = bytes32(uint256(1));
    bytes32 internal constant INITIAL_ADMISSION_STATUS_SLOT =
        0x375f13b0f395f58180c4440e3093a15026ebe690dc75ff0edddf4387bd26fae6;

    MeasurementRegistry internal registry;

    event AdmissionStatusChanged(
        bytes32 indexed admissionId,
        MeasurementRegistry.Status previousStatus,
        MeasurementRegistry.Status newStatus,
        uint64 indexed policyRevision
    );

    event PolicyUpdated(
        uint64 indexed policyRevision, bytes32 indexed activePolicyHash, uint256 accepted, uint256 deprecated
    );

    function setUp() public {
        registry = new MeasurementRegistry();
        _initializeRegistry();
    }

    function test_GenesisStorageLoadsExpectedState() public view {
        assertEq(registry.AUTHORITY(), AUTHORITY);
        assertEq(registry.bootstrapPolicyHash(), BOOTSTRAP_POLICY_HASH);
        assertEq(registry.activePolicyHash(), BOOTSTRAP_POLICY_HASH);
        assertEq(registry.policyRevision(), 1);
        assertEq(registry.acceptedCount(), 1);
        assertTrue(registry.isAccepted(INITIAL_ADMISSION_ID));
        assertEq(uint8(registry.statusOf(INITIAL_ADMISSION_ID)), uint8(MeasurementRegistry.Status.Accepted));
        assertEq(uint8(registry.statusOf(keccak256("unknown"))), uint8(MeasurementRegistry.Status.Unknown));
    }

    function test_StorageSlotGoldenValues() public pure {
        assertEq(keccak256(abi.encode(INITIAL_ADMISSION_ID, REGISTRY_STORAGE_LOCATION)), INITIAL_ADMISSION_STATUS_SLOT);
        assertEq(
            bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 1),
            0xa3ae60943e4f183142036d77b94858085814dd428f131289aea7e42703fb0b01
        );
        assertEq(
            bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 2),
            0xa3ae60943e4f183142036d77b94858085814dd428f131289aea7e42703fb0b02
        );
        assertEq(
            bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 3),
            0xa3ae60943e4f183142036d77b94858085814dd428f131289aea7e42703fb0b03
        );
        assertEq(
            bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 4),
            0xa3ae60943e4f183142036d77b94858085814dd428f131289aea7e42703fb0b04
        );
    }

    function test_StorageLocationMatchesERC7201Derivation() public pure {
        bytes32 derivedLocation = keccak256(abi.encode(uint256(keccak256("seismic.storage.MeasurementRegistry")) - 1))
            & ~bytes32(uint256(0xff));

        assertEq(REGISTRY_STORAGE_LOCATION, derivedLocation);
    }

    function test_AcceptsUnknownAdmissionId() public {
        bytes32 admissionId = keccak256("new-admission");
        bytes32 newPolicyHash = keccak256("policy-2");

        _apply(_singleton(admissionId), _empty(), newPolicyHash);

        assertTrue(registry.isAccepted(admissionId));
        assertEq(registry.acceptedCount(), 2);
        assertEq(registry.policyRevision(), 2);
        assertEq(registry.activePolicyHash(), newPolicyHash);
        assertEq(registry.bootstrapPolicyHash(), BOOTSTRAP_POLICY_HASH);
    }

    function test_ReinstatesDeprecatedAdmissionId() public {
        bytes32 admissionId = keccak256("deprecated-admission");
        _storeStatus(admissionId, MeasurementRegistry.Status.Deprecated);

        _apply(_singleton(admissionId), _empty(), keccak256("policy-2"));

        assertTrue(registry.isAccepted(admissionId));
        assertEq(registry.acceptedCount(), 2);
    }

    function test_DeprecatesAcceptedAdmissionId() public {
        _apply(_empty(), _singleton(INITIAL_ADMISSION_ID), keccak256("policy-2"));

        assertFalse(registry.isAccepted(INITIAL_ADMISSION_ID));
        assertEq(uint8(registry.statusOf(INITIAL_ADMISSION_ID)), uint8(MeasurementRegistry.Status.Deprecated));
        assertEq(registry.acceptedCount(), 0);
    }

    function test_AppliesMixedUpdateAtomically() public {
        bytes32 added = keccak256("added");

        _apply(_singleton(added), _singleton(INITIAL_ADMISSION_ID), keccak256("policy-2"));

        assertTrue(registry.isAccepted(added));
        assertFalse(registry.isAccepted(INITIAL_ADMISSION_ID));
        assertEq(registry.acceptedCount(), 1);
        assertEq(registry.policyRevision(), 2);
    }

    function test_EmitsStatusChangesBeforePolicyUpdate() public {
        bytes32 added = keccak256("added");
        bytes32 newPolicyHash = keccak256("policy-2");

        vm.expectEmit(true, true, false, true, address(registry));
        emit AdmissionStatusChanged(added, MeasurementRegistry.Status.Unknown, MeasurementRegistry.Status.Accepted, 2);
        vm.expectEmit(true, true, false, true, address(registry));
        emit AdmissionStatusChanged(
            INITIAL_ADMISSION_ID, MeasurementRegistry.Status.Accepted, MeasurementRegistry.Status.Deprecated, 2
        );
        vm.expectEmit(true, true, false, true, address(registry));
        emit PolicyUpdated(2, newPolicyHash, 1, 1);

        _apply(_singleton(added), _singleton(INITIAL_ADMISSION_ID), newPolicyHash);
    }

    function test_RevertWhen_CallerIsNotAuthority() public {
        address caller = makeAddr("caller");

        vm.prank(caller);
        vm.expectRevert(abi.encodeWithSelector(MeasurementRegistry.Unauthorized.selector, caller));
        registry.applyPolicyUpdate(_singleton(keccak256("added")), _empty(), keccak256("policy-2"));
    }

    function test_RevertWhen_RegistryIsUninitialized() public {
        MeasurementRegistry uninitializedRegistry = new MeasurementRegistry();

        vm.prank(AUTHORITY);
        vm.expectRevert(MeasurementRegistry.Uninitialized.selector);
        uninitializedRegistry.applyPolicyUpdate(_singleton(keccak256("added")), _empty(), keccak256("policy-2"));
    }

    function test_RevertWhen_UpdateIsEmpty() public {
        vm.prank(AUTHORITY);
        vm.expectRevert(MeasurementRegistry.EmptyUpdate.selector);
        registry.applyPolicyUpdate(_empty(), _empty(), keccak256("policy-2"));
    }

    function test_RevertWhen_PolicyHashIsUnchanged() public {
        vm.prank(AUTHORITY);
        vm.expectRevert(abi.encodeWithSelector(MeasurementRegistry.UnchangedPolicyHash.selector, BOOTSTRAP_POLICY_HASH));
        registry.applyPolicyUpdate(_singleton(keccak256("added")), _empty(), BOOTSTRAP_POLICY_HASH);
    }

    function test_RevertWhen_AcceptContainsDuplicate() public {
        bytes32 admissionId = keccak256("duplicate");
        bytes32[] memory accept = new bytes32[](2);
        accept[0] = admissionId;
        accept[1] = admissionId;

        vm.prank(AUTHORITY);
        vm.expectRevert(abi.encodeWithSelector(MeasurementRegistry.DuplicateAdmissionId.selector, admissionId));
        registry.applyPolicyUpdate(accept, _empty(), keccak256("policy-2"));
    }

    function test_RevertWhen_DeprecateContainsDuplicate() public {
        bytes32[] memory deprecate = new bytes32[](2);
        deprecate[0] = INITIAL_ADMISSION_ID;
        deprecate[1] = INITIAL_ADMISSION_ID;

        vm.prank(AUTHORITY);
        vm.expectRevert(abi.encodeWithSelector(MeasurementRegistry.DuplicateAdmissionId.selector, INITIAL_ADMISSION_ID));
        registry.applyPolicyUpdate(_empty(), deprecate, keccak256("policy-2"));
    }

    function test_RevertWhen_BatchesContradict() public {
        vm.prank(AUTHORITY);
        vm.expectRevert(
            abi.encodeWithSelector(MeasurementRegistry.ContradictoryAdmissionId.selector, INITIAL_ADMISSION_ID)
        );
        registry.applyPolicyUpdate(
            _singleton(INITIAL_ADMISSION_ID), _singleton(INITIAL_ADMISSION_ID), keccak256("policy-2")
        );
    }

    function test_RevertWhen_AcceptingAcceptedAdmissionId() public {
        vm.prank(AUTHORITY);
        vm.expectRevert(
            abi.encodeWithSelector(
                MeasurementRegistry.InvalidTransition.selector,
                INITIAL_ADMISSION_ID,
                MeasurementRegistry.Status.Accepted,
                MeasurementRegistry.Status.Accepted
            )
        );
        registry.applyPolicyUpdate(_singleton(INITIAL_ADMISSION_ID), _empty(), keccak256("policy-2"));
    }

    function test_RevertWhen_DeprecatingUnknownAdmissionId() public {
        bytes32 admissionId = keccak256("unknown");

        vm.prank(AUTHORITY);
        vm.expectRevert(
            abi.encodeWithSelector(
                MeasurementRegistry.InvalidTransition.selector,
                admissionId,
                MeasurementRegistry.Status.Unknown,
                MeasurementRegistry.Status.Deprecated
            )
        );
        registry.applyPolicyUpdate(_empty(), _singleton(admissionId), keccak256("policy-2"));
    }

    function test_RevertWhen_DeprecatingDeprecatedAdmissionId() public {
        bytes32 admissionId = keccak256("deprecated");
        _storeStatus(admissionId, MeasurementRegistry.Status.Deprecated);

        vm.prank(AUTHORITY);
        vm.expectRevert(
            abi.encodeWithSelector(
                MeasurementRegistry.InvalidTransition.selector,
                admissionId,
                MeasurementRegistry.Status.Deprecated,
                MeasurementRegistry.Status.Deprecated
            )
        );
        registry.applyPolicyUpdate(_empty(), _singleton(admissionId), keccak256("policy-2"));
    }

    function test_RevertRollsBackEarlierBatchEntries() public {
        bytes32 validAdmissionId = keccak256("valid");
        bytes32[] memory accept = new bytes32[](2);
        accept[0] = validAdmissionId;
        accept[1] = INITIAL_ADMISSION_ID;

        vm.prank(AUTHORITY);
        vm.expectRevert(
            abi.encodeWithSelector(
                MeasurementRegistry.InvalidTransition.selector,
                INITIAL_ADMISSION_ID,
                MeasurementRegistry.Status.Accepted,
                MeasurementRegistry.Status.Accepted
            )
        );
        registry.applyPolicyUpdate(accept, _empty(), keccak256("policy-2"));

        assertFalse(registry.isAccepted(validAdmissionId));
        assertEq(registry.acceptedCount(), 1);
        assertEq(registry.policyRevision(), 1);
        assertEq(registry.activePolicyHash(), BOOTSTRAP_POLICY_HASH);
    }

    function testFuzz_AcceptThenDeprecate(bytes32 admissionId) public {
        vm.assume(admissionId != INITIAL_ADMISSION_ID);

        _apply(_singleton(admissionId), _empty(), keccak256("policy-2"));
        _apply(_empty(), _singleton(admissionId), keccak256("policy-3"));

        assertFalse(registry.isAccepted(admissionId));
        assertEq(uint8(registry.statusOf(admissionId)), uint8(MeasurementRegistry.Status.Deprecated));
        assertEq(registry.acceptedCount(), 1);
        assertEq(registry.policyRevision(), 3);
    }

    function _initializeRegistry() internal {
        vm.store(address(registry), bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 1), BOOTSTRAP_POLICY_HASH);
        vm.store(address(registry), bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 2), BOOTSTRAP_POLICY_HASH);
        vm.store(address(registry), bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 3), bytes32(uint256(1)));
        vm.store(address(registry), bytes32(uint256(REGISTRY_STORAGE_LOCATION) + 4), bytes32(uint256(1)));
        _storeStatus(INITIAL_ADMISSION_ID, MeasurementRegistry.Status.Accepted);
    }

    function _storeStatus(bytes32 admissionId, MeasurementRegistry.Status status) internal {
        bytes32 slot = keccak256(abi.encode(admissionId, REGISTRY_STORAGE_LOCATION));
        vm.store(address(registry), slot, bytes32(uint256(uint8(status))));
    }

    function _apply(bytes32[] memory accept, bytes32[] memory deprecate, bytes32 newPolicyHash) internal {
        vm.prank(AUTHORITY);
        registry.applyPolicyUpdate(accept, deprecate, newPolicyHash);
    }

    function _singleton(bytes32 value) internal pure returns (bytes32[] memory values) {
        values = new bytes32[](1);
        values[0] = value;
    }

    function _empty() internal pure returns (bytes32[] memory values) {
        values = new bytes32[](0);
    }
}
