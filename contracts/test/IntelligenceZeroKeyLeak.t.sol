// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import {MockSRC20} from "./utils/MockSRC20.sol";
import {Directory} from "../src/directory/Directory.sol";
import {Intelligence} from "../src/intelligence/Intelligence.sol";
import {IDirectory} from "seismic-std-lib/interfaces/IDirectory.sol";
import {IIntelligence} from "seismic-std-lib/interfaces/IIntelligence.sol";

/// @notice Zellic 3.4 — an unkeyed Intelligence provider leaks SRC20 amounts.
///         `Directory.encrypt` loaded `keys[to]` even when unset (`suint256(0)`),
///         so a provider with no registered key produced AES-256-GCM ciphertext
///         under a ZERO key with `keyHash == keccak256(uint256(0))` (a public
///         marker). Any fresh zero-key address could `Directory.decrypt` the
///         emitted Transfer/Approval payload and recover the amount.
///
///         These tests assert the SECURE post-fix behavior: unkeyed providers
///         are rejected, zero keys are rejected at the Directory layer, and no
///         emitted event carries a zero-key-recoverable ciphertext.
contract IntelligenceZeroKeyLeakTest is Test {
    MockSRC20 token;

    address constant INTELLIGENCE_ADDR = 0x1000000000000000000000000000000000000005;
    address constant DIRECTORY_ADDR = 0x1000000000000000000000000000000000000004;

    IDirectory directory = IDirectory(DIRECTORY_ADDR);
    IIntelligence intelligence = IIntelligence(INTELLIGENCE_ADDR);

    address sender = makeAddr("sender");
    address recipient = makeAddr("recipient");
    address provider = makeAddr("provider");
    address eve = makeAddr("eve");

    bytes32 constant TRANSFER_TOPIC = keccak256("Transfer(address,address,bytes32,bytes)");

    /// @dev Public marker hash emitted when encrypting under the zero key.
    bytes32 constant ZERO_KEY_HASH = keccak256(abi.encodePacked(uint256(0)));

    function setUp() public {
        deployCodeTo("Directory.sol", DIRECTORY_ADDR);
        deployCodeTo("Intelligence.sol", INTELLIGENCE_ADDR);

        token = new MockSRC20("Token", "TKN", 18);
    }

    /*//////////////////////////////////////////////////////////////
                    ADD PROVIDER REQUIRES A KEY
    //////////////////////////////////////////////////////////////*/

    /// @dev Pre-fix: `addProvider` never checked for a key, so a zero-key
    ///      provider could be registered. Post-fix it must revert.
    function test_RevertWhen_AddProviderWithoutKey() public {
        assertFalse(directory.checkHasKey(provider));

        vm.prank(intelligence.INITIAL_OWNER());
        vm.expectRevert("PROVIDER_NO_KEY");
        intelligence.addProvider(provider);

        assertEq(intelligence.numProviders(), 0);
    }

    function test_AddProviderSucceedsWhenProviderHasKey() public {
        _registerKey(provider);

        vm.prank(intelligence.INITIAL_OWNER());
        intelligence.addProvider(provider);

        assertEq(intelligence.numProviders(), 1);
    }

    /*//////////////////////////////////////////////////////////////
                    ZERO-KEY LEAK IS UNREACHABLE
    //////////////////////////////////////////////////////////////*/

    /// @dev Faithful adaptation of PoC §6.4, flipped to the secure expectation.
    ///      Pre-fix the unkeyed provider registers and its Transfer log carries
    ///      `keyHash == keccak256(uint256(0))` with non-empty ciphertext that
    ///      `eve` (a fresh zero-key address) can decrypt back to the amount.
    ///      Post-fix registration reverts, so no such log is ever emitted.
    function test_UnkeyedProviderAmountNotRecoverableByFreshAddress() public {
        // Pre-fix: succeeds and plants the leak. Post-fix: reverts, leak absent.
        vm.prank(intelligence.INITIAL_OWNER());
        try intelligence.addProvider(provider) {} catch {}

        token.mint(sender, suint256(1e18));

        vm.recordLogs();
        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));

        Vm.Log[] memory logs = _getTransferLogs();

        bool leaked = false;
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[3] != ZERO_KEY_HASH) {
                continue;
            }
            bytes memory data = abi.decode(logs[i].data, (bytes));
            if (data.length == 0) {
                continue;
            }
            // A zero-key ciphertext: a fresh zero-key address recovers the amount.
            vm.prank(eve);
            uint256 recovered = abi.decode(directory.decrypt(data), (uint256));
            if (recovered == 1e18) {
                leaked = true;
            }
        }

        assertFalse(leaked, "unkeyed provider leaked a zero-key-recoverable amount");
    }

    /*//////////////////////////////////////////////////////////////
                    DIRECTORY DEFENSE IN DEPTH
    //////////////////////////////////////////////////////////////*/

    function test_RevertWhen_SetKeyZero() public {
        vm.prank(sender);
        vm.expectRevert("ZERO_KEY");
        directory.setKey(suint256(0));

        assertFalse(directory.checkHasKey(sender));
    }

    function test_RevertWhen_EncryptToZeroKeyAddress() public {
        assertFalse(directory.checkHasKey(provider));

        vm.expectRevert("ZERO_KEY");
        directory.encrypt(provider, abi.encodePacked(uint256(1e18)));
    }

    /*//////////////////////////////////////////////////////////////
                            HELPERS
    //////////////////////////////////////////////////////////////*/

    function _registerKey(address user) internal {
        vm.prank(user);
        directory.setKey(suint256(0xABC));
    }

    function _getTransferLogs() internal returns (Vm.Log[] memory) {
        Vm.Log[] memory allLogs = vm.getRecordedLogs();

        uint256 count = 0;
        for (uint256 i = 0; i < allLogs.length; i++) {
            if (allLogs[i].topics[0] == TRANSFER_TOPIC) {
                count++;
            }
        }

        Vm.Log[] memory transferLogs = new Vm.Log[](count);
        uint256 idx = 0;
        for (uint256 i = 0; i < allLogs.length; i++) {
            if (allLogs[i].topics[0] == TRANSFER_TOPIC) {
                transferLogs[idx++] = allLogs[i];
            }
        }

        return transferLogs;
    }
}
