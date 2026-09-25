// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {MeasurementRegistry} from "./MeasurementRegistry.sol";

/// @title MeasurementAuthorityDev
/// @notice Single-owner forwarder that drives `MeasurementRegistry.applyPolicyUpdate`.
/// @dev FOR DEVELOPMENT AND TESTS ONLY. The owner is a well-known Anvil test key,
///      so any network running this contract has a publicly known policy authority.
///      It exists so local networks and integration tests can exercise live policy
///      mutation (accept, deprecate, reinstate) against the registry with one
///      transaction from one key.
///
///      The registry recognizes one fixed authority address
///      (`MeasurementRegistry.AUTHORITY`), which no EOA can occupy, so even a
///      development network needs a forwarding contract there. Production networks
///      install an audited multisig/governance component behind the same address
///      at genesis; the registry and the address itself never change.
contract MeasurementAuthorityDev {
    /// @notice The only account allowed to apply policy updates (Anvil account 0).
    address public constant OWNER = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    /// @notice The registry this authority administers, installed at genesis.
    MeasurementRegistry public constant REGISTRY = MeasurementRegistry(0x1000000000000000000000000000000000000001);

    error NotOwner(address caller);

    /// @notice Forwards a policy update to the registry.
    /// @dev The registry enforces payload validity (status transitions, duplicates,
    ///      policy-hash change) and emits all policy events.
    /// @param accept IDs transitioning from Unknown or Deprecated to Accepted.
    /// @param deprecate IDs transitioning from Accepted to Deprecated.
    /// @param newActivePolicyHash SHA-256 of the exact complete new policy document bytes.
    function applyPolicyUpdate(bytes32[] calldata accept, bytes32[] calldata deprecate, bytes32 newActivePolicyHash)
        external
    {
        if (msg.sender != OWNER) revert NotOwner(msg.sender);

        REGISTRY.applyPolicyUpdate(accept, deprecate, newActivePolicyHash);
    }
}
