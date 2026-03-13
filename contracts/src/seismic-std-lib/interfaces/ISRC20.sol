// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

/// @notice Interface for SRC20 — ERC20 with confidential (shielded) balances.
interface ISRC20 {
    event Transfer(address indexed from, address indexed to, bytes32 indexed encryptKeyHash, bytes encryptedAmount);

    event Approval(
        address indexed owner, address indexed spender, bytes32 indexed encryptKeyHash, bytes encryptedAmount
    );

    function name() external view returns (string memory);

    function symbol() external view returns (string memory);

    function decimals() external view returns (uint8);

    function nonces(address owner) external view returns (uint256);

    function approve(address spender, suint256 amount) external returns (bool);

    function transfer(address to, suint256 amount) external returns (bool);

    function transferFrom(address from, address to, suint256 amount) external returns (bool);

    function permit(address owner, address spender, suint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external;

    function DOMAIN_SEPARATOR() external view returns (bytes32);

    function balanceOfSigned(address owner, uint256 expiry, bytes calldata signature) external view returns (uint256);

    function balance() external view returns (uint256);

    function allowance(address spender) external view returns (uint256);
}
