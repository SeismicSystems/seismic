// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

interface ISRC20 {
    function balanceOfSigned(address owner, uint256 expiry, bytes calldata signature) external view returns (uint256);
}
