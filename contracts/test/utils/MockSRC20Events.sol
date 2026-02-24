// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

/// @title MockSRC20Events
/// @notice Test-only contract that emits SRC20-style Transfer and Approval
///         events with encrypted amounts.  Used by the Python SDK integration
///         tests to exercise the event watcher without requiring a full
///         SRC20 + Intelligence integration.
contract MockSRC20Events {
    event Transfer(address indexed from, address indexed to, bytes32 indexed encryptKeyHash, bytes encryptedAmount);

    event Approval(
        address indexed owner, address indexed spender, bytes32 indexed encryptKeyHash, bytes encryptedAmount
    );

    function emitTransfer(address from, address to, bytes32 encryptKeyHash, bytes calldata encryptedAmount) external {
        emit Transfer(from, to, encryptKeyHash, encryptedAmount);
    }

    function emitApproval(address owner, address spender, bytes32 encryptKeyHash, bytes calldata encryptedAmount)
        external
    {
        emit Approval(owner, spender, encryptKeyHash, encryptedAmount);
    }
}
