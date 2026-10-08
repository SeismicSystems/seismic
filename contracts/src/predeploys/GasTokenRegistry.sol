// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title GasTokenRegistry
/// @notice An owner-managed, ordered registry of tokens eligible for gas payment.
/// @dev Intended genesis predeploy address: 0x0000000000000000000000476173546f6b656e73
///      (ASCII "GasTokens"). This contract stores configuration only; execution-client
///      integration is required before registered tokens can pay gas.
///
///      Tokens are appended in payment-priority order and cannot be removed or
///      reordered. Their address, balance mapping slot, storage mode, and decimals
///      cannot be changed after registration; only their active flag can change.
///      Tokens must use 0–18 decimals and a compatible mapping from address to a
///      full-width balance: suint256 for Shielded, uint256 for Public. Decimals are
///      owner-supplied metadata, never discovered by executing token code here.
///      The owner verifies layout, precision, direct fee accounting, and suitability
///      under the fixed conversion of one whole token per whole native unit,
///      including after proxy upgrades. Register the proxy, not its implementation.
///
///      Execution clients will read public storage directly. Declaration order and
///      struct packing must therefore remain stable:
///        slot 0                     : owner (address)
///        slot 1                     : tokens.length
///        keccak256(abi.encode(1))+2*i: tokens[i].token (low 160 bits),
///                                     tokens[i].active (byte offset 20),
///                                     tokens[i].balanceStorageMode (byte offset 21;
///                                       0 = Shielded, 1 = Public),
///                                     tokens[i].decimals (byte offset 22)
///        preceding slot + 1         : tokens[i].balanceSlot (uint256)
///      The balance mapping key is keccak256(abi.encode(sender, balanceSlot)).
///
///      Genesis installation bypasses the constructor. Deployment tooling must
///      seed owner at slot 0 with the same initial owner as ProtocolParams; normal
///      deployments use msg.sender, matching ProtocolParams' authorization model.
contract GasTokenRegistry {
    /// @notice Storage semantics of the token's balance mapping, not its other fields.
    /// @dev These numeric values are part of the execution-client storage interface.
    enum BalanceStorageMode {
        Shielded,
        Public
    }

    struct GasToken {
        address token;
        bool active;
        BalanceStorageMode balanceStorageMode;
        uint8 decimals;
        uint256 balanceSlot;
    }

    /// @notice Maximum number of entries, including inactive tokens, to bound scans.
    uint256 public constant MAX_TOKENS = 32;

    /// @notice Maximum supported token precision; zero decimals is valid.
    uint8 public constant MAX_DECIMALS = 18;

    /// @notice Registry owner. Slot 0; ownership is independent of ProtocolParams.
    address public owner;

    /// @notice Append-only entries in payment-priority order. Slot 1.
    GasToken[] public tokens;

    event TokenAdded(
        uint256 indexed index,
        address indexed token,
        uint256 balanceSlot,
        BalanceStorageMode balanceStorageMode,
        uint8 decimals
    );
    event TokenActivationChanged(uint256 indexed index, address indexed token, bool active);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    error OnlyOwner();
    error ZeroAddress();
    error TokenHasNoCode(address token);
    error TokenAlreadyRegistered(address token);
    error RegistryFull();
    error UnsupportedDecimals(uint8 decimals);
    error TokenNotRegistered(address token);

    modifier onlyOwner() {
        if (msg.sender != owner) revert OnlyOwner();
        _;
    }

    /// @notice Sets the deployer as owner for normal deployments; not run at genesis.
    constructor() {
        owner = msg.sender;
        emit OwnershipTransferred(address(0), msg.sender);
    }

    /// @notice Appends a token as active, at the lowest registered-token priority.
    /// @param token The token address, or proxy address for an upgradeable token.
    /// @param balanceSlot Storage position of the token's balance mapping; zero is valid.
    /// @param balanceStorageMode Whether balance writes must use shielded or public storage.
    /// @param decimals Verified token base-unit precision, from zero through eighteen.
    /// @return index The permanent index of the new entry.
    function addToken(address token, uint256 balanceSlot, BalanceStorageMode balanceStorageMode, uint8 decimals)
        external
        onlyOwner
        returns (uint256 index)
    {
        if (token == address(0)) revert ZeroAddress();
        if (token.code.length == 0) revert TokenHasNoCode(token);
        if (decimals > MAX_DECIMALS) revert UnsupportedDecimals(decimals);

        uint256 count = tokens.length;
        for (uint256 i = 0; i < count; i++) {
            if (tokens[i].token == token) revert TokenAlreadyRegistered(token);
        }
        if (count >= MAX_TOKENS) revert RegistryFull();

        index = count;
        tokens.push(
            GasToken({
                token: token,
                active: true,
                balanceStorageMode: balanceStorageMode,
                decimals: decimals,
                balanceSlot: balanceSlot
            })
        );
        emit TokenAdded(index, token, balanceSlot, balanceStorageMode, decimals);
    }

    /// @notice Activates a registered token without changing its original payment priority.
    /// @dev Idempotent: activating an active token is allowed.
    /// @param token The registered token address, or proxy address for an upgradeable token.
    function activateToken(address token) external onlyOwner {
        _setTokenActive(token, true);
    }

    /// @notice Deactivates a registered token without removing it or changing its metadata.
    /// @dev Idempotent: deactivating an inactive token is allowed; token code is not checked.
    /// @param token The registered token address, or proxy address for an upgradeable token.
    function deactivateToken(address token) external onlyOwner {
        _setTokenActive(token, false);
    }

    /// @notice Returns the total number of registered entries, including inactive ones.
    function tokenCount() external view returns (uint256) {
        return tokens.length;
    }

    /// @notice Transfers ownership, matching ProtocolParams' authorization model.
    function transferOwnership(address newOwner) external onlyOwner {
        if (newOwner == address(0)) revert ZeroAddress();
        address previousOwner = owner;
        owner = newOwner;
        emit OwnershipTransferred(previousOwner, newOwner);
    }

    /// @notice Permanently disables administration; existing token configuration remains.
    function renounceOwnership() external onlyOwner {
        address previousOwner = owner;
        owner = address(0);
        emit OwnershipTransferred(previousOwner, address(0));
    }

    function _setTokenActive(address token, bool active) internal {
        uint256 count = tokens.length;
        for (uint256 i = 0; i < count; i++) {
            GasToken storage entry = tokens[i];
            if (entry.token == token) {
                entry.active = active;
                emit TokenActivationChanged(i, token, active);
                return;
            }
        }
        revert TokenNotRegistered(token);
    }
}
