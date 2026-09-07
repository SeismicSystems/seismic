// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, Vm} from "forge-std/Test.sol";
import {KeyRotationRegistry} from "../src/enclave/KeyRotationRegistry.sol";

contract KeyRotationRegistryTest is Test {
    KeyRotationRegistry public registry;

    address public admin;
    address public alice;

    uint64 public constant MIN_DELAY = 32;

    event RotationAnnounced(uint64 indexed epoch, uint64 activationBlock);
    event AdminTransferred(address indexed previousAdmin, address indexed newAdmin);

    function setUp() public {
        admin = address(this);
        alice = makeAddr("alice");

        registry = new KeyRotationRegistry(admin, MIN_DELAY);
    }

    // ============ Constructor Tests ============

    function test_ConstructorSetsAdminAndDelay() public view {
        assertEq(registry.admin(), admin);
        assertEq(registry.minActivationDelay(), MIN_DELAY);
        assertEq(registry.rotationCount(), 0);
        assertEq(registry.currentEpoch(), 0);
    }

    function test_ConstructorRejectsZeroAdmin() public {
        vm.expectRevert(KeyRotationRegistry.ZeroAddress.selector);
        new KeyRotationRegistry(address(0), MIN_DELAY);
    }

    function test_ConstructorEmitsAdminTransferred() public {
        vm.expectEmit(true, true, false, false);
        emit AdminTransferred(address(0), admin);
        new KeyRotationRegistry(admin, MIN_DELAY);
    }

    // ============ announceRotation Tests ============

    function test_AnnounceRotation() public {
        vm.roll(100);
        uint64 activation = 200;

        vm.expectEmit(true, false, false, true);
        emit RotationAnnounced(1, activation);
        registry.announceRotation(activation);

        assertEq(registry.rotationCount(), 1);
        (uint64 epoch, uint64 activationBlock, uint64 announcedAtBlock) = registry.rotations(0);
        assertEq(epoch, 1);
        assertEq(activationBlock, activation);
        assertEq(announcedAtBlock, 100);
    }

    function test_AnnounceRotationOnlyAdmin() public {
        vm.roll(100);
        vm.prank(alice);
        vm.expectRevert(KeyRotationRegistry.OnlyAdmin.selector);
        registry.announceRotation(200);
    }

    function test_AnnounceRotationFailsClosedWithoutDelay() public {
        // A genesis placement that forgot to write slot 2 must refuse rotations.
        KeyRotationRegistry unconfigured = new KeyRotationRegistry(admin, 0);
        vm.roll(100);
        vm.expectRevert(KeyRotationRegistry.DelayNotConfigured.selector);
        unconfigured.announceRotation(type(uint64).max);
    }

    function test_AnnounceRotationEnforcesMinDelay() public {
        vm.roll(100);

        // One block short of the floor reverts...
        vm.expectRevert(
            abi.encodeWithSelector(KeyRotationRegistry.ActivationTooSoon.selector, 100 + MIN_DELAY - 1, 100 + MIN_DELAY)
        );
        registry.announceRotation(uint64(100 + MIN_DELAY - 1));

        // ...exactly at the floor passes.
        registry.announceRotation(uint64(100 + MIN_DELAY));
        assertEq(registry.rotationCount(), 1);
    }

    function test_AnnounceRotationRejectsSecondPending() public {
        vm.roll(100);
        registry.announceRotation(200);

        // Still pending at the block before activation.
        vm.roll(199);
        vm.expectRevert(abi.encodeWithSelector(KeyRotationRegistry.RotationPending.selector, 1, 200));
        registry.announceRotation(400);

        // At the activation block the rotation is active, so a new one may be
        // announced.
        vm.roll(200);
        registry.announceRotation(uint64(200 + MIN_DELAY));
        assertEq(registry.rotationCount(), 2);
    }

    function test_EpochsAreDenseAndActivationsIncrease() public {
        vm.roll(100);
        registry.announceRotation(200);
        vm.roll(200);
        registry.announceRotation(300);
        vm.roll(300);
        registry.announceRotation(400);

        for (uint256 i = 0; i < 3; i++) {
            (uint64 epoch, uint64 activationBlock,) = registry.rotations(i);
            assertEq(epoch, uint64(i) + 1);
            assertEq(activationBlock, 200 + uint64(i) * 100);
        }
    }

    // ============ View Tests ============

    function test_EpochFlipsExactlyAtActivation() public {
        vm.roll(100);
        registry.announceRotation(200);

        assertEq(registry.epochAtBlock(0), 0);
        assertEq(registry.epochAtBlock(199), 0);
        assertEq(registry.epochAtBlock(200), 1);
        assertEq(registry.epochAtBlock(type(uint64).max), 1);

        vm.roll(199);
        assertEq(registry.currentEpoch(), 0);
        vm.roll(200);
        assertEq(registry.currentEpoch(), 1);
    }

    function test_PendingRotationTracksTheWindow() public {
        (bool exists,,) = registry.pendingRotation();
        assertFalse(exists);

        vm.roll(100);
        registry.announceRotation(200);

        vm.roll(150);
        (bool pendingExists, uint64 epoch, uint64 activationBlock) = registry.pendingRotation();
        assertTrue(pendingExists);
        assertEq(epoch, 1);
        assertEq(activationBlock, 200);

        vm.roll(200);
        (exists,,) = registry.pendingRotation();
        assertFalse(exists);
    }

    function test_MultiRotationEpochAtBlock() public {
        vm.roll(100);
        registry.announceRotation(200);
        vm.roll(200);
        registry.announceRotation(300);

        assertEq(registry.epochAtBlock(199), 0);
        assertEq(registry.epochAtBlock(200), 1);
        assertEq(registry.epochAtBlock(299), 1);
        assertEq(registry.epochAtBlock(300), 2);
    }

    // ============ transferAdmin Tests ============

    function test_TransferAdmin() public {
        vm.expectEmit(true, true, false, false);
        emit AdminTransferred(admin, alice);
        registry.transferAdmin(alice);
        assertEq(registry.admin(), alice);

        // The old admin loses the announce authority; the new one has it.
        vm.roll(100);
        vm.expectRevert(KeyRotationRegistry.OnlyAdmin.selector);
        registry.announceRotation(200);
        vm.prank(alice);
        registry.announceRotation(200);
    }

    function test_TransferAdminRejectsZeroAddress() public {
        vm.expectRevert(KeyRotationRegistry.ZeroAddress.selector);
        registry.transferAdmin(address(0));
    }

    function test_TransferAdminOnlyAdmin() public {
        vm.prank(alice);
        vm.expectRevert(KeyRotationRegistry.OnlyAdmin.selector);
        registry.transferAdmin(alice);
    }

    // ============ Raw Layout Golden Tests (cross-repo lockstep) ============
    //
    // seismic-reth never calls this contract: it reads these raw slots and filters
    // logs by the raw topic (`reth-seismic-keys`'s `registry` module). These tests
    // pin the exact bytes that side decodes; if any of them changes, the reth
    // mirror must change in the same coordinated release.

    function test_Golden_AdminLivesInSlotZero() public view {
        bytes32 slot0 = vm.load(address(registry), bytes32(uint256(0)));
        assertEq(address(uint160(uint256(slot0))), admin);
    }

    function test_Golden_RotationsLengthLivesInSlotOne() public {
        assertEq(uint256(vm.load(address(registry), bytes32(uint256(1)))), 0);

        vm.roll(100);
        registry.announceRotation(200);
        assertEq(uint256(vm.load(address(registry), bytes32(uint256(1)))), 1);
    }

    function test_Golden_MinDelayLivesInSlotTwo() public view {
        bytes32 slot2 = vm.load(address(registry), bytes32(uint256(2)));
        assertEq(uint256(slot2), uint256(MIN_DELAY));
    }

    /// @dev Mirrors the reth-side decode fixture byte for byte: announcing at
    ///      block 100 with activation 200 must store exactly
    ///      `announced (0x64) | activation (0xC8) | epoch (0x01)` packed low-order
    ///      first at `keccak256(uint256(1))`.
    function test_Golden_EntryPackingMatchesRethFixture() public {
        vm.roll(100);
        registry.announceRotation(200);

        bytes32 elementSlot = keccak256(abi.encode(uint256(1)));
        assertEq(
            elementSlot,
            // keccak256(uint256(1)), pinned in reth-seismic-keys' slot test too.
            bytes32(0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6)
        );
        assertEq(
            vm.load(address(registry), elementSlot),
            bytes32(0x0000000000000000000000000000006400000000000000c80000000000000001)
        );
    }

    function test_Golden_SecondEntryAtBasePlusOne() public {
        vm.roll(100);
        registry.announceRotation(200);
        vm.roll(200);
        registry.announceRotation(300);

        bytes32 base = keccak256(abi.encode(uint256(1)));
        bytes32 word = vm.load(address(registry), bytes32(uint256(base) + 1));
        // announced 200 (0xC8) | activation 300 (0x12C) | epoch 2
        assertEq(word, bytes32(0x000000000000000000000000000000c8000000000000012c0000000000000002));
    }

    /// @dev The event signature string is consensus-frozen: reth filters on
    ///      `keccak256("RotationAnnounced(uint64,uint64)")` as topic0.
    function test_Golden_EventTopicMatchesFrozenSignature() public {
        vm.roll(100);
        vm.recordLogs();
        registry.announceRotation(200);

        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1);
        assertEq(logs[0].topics[0], keccak256("RotationAnnounced(uint64,uint64)"));
        // epoch is the sole indexed param; activationBlock rides in the data.
        assertEq(logs[0].topics[1], bytes32(uint256(1)));
        assertEq(abi.decode(logs[0].data, (uint256)), 200);
    }
}
