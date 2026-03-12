// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import {ISRC20} from "./interfaces/ISRC20.sol";

/// @notice Batch reader for SRC20 shielded balances with multiple implementation approaches.
contract SRC20Multicall {
    struct BalanceResult {
        address token;
        uint256 balance;
        bool success;
    }

    /// @notice Batch read balances with detailed results including success status
    function batchBalancesDetailed(address owner, address[] calldata tokens, uint256 expiry, bytes calldata signature)
        external
        view
        returns (BalanceResult[] memory results)
    {
        results = new BalanceResult[](tokens.length);

        for (uint256 i = 0; i < tokens.length;) {
            results[i].token = tokens[i];

            bytes memory data =
                abi.encodeWithSignature("balanceOfSigned(address,uint256,bytes)", owner, expiry, signature);

            (bool success, bytes memory returnData) = tokens[i].staticcall(data);

            results[i].success = success;
            if (success && returnData.length >= 32) {
                results[i].balance = abi.decode(returnData, (uint256));
            } else {
                results[i].balance = 0;
            }

            unchecked {
                ++i;
            }
        }
    }

    /// @notice Batch read balances using interface calls (original approach)
    function batchBalancesInterface(address owner, address[] calldata tokens, uint256 expiry, bytes calldata signature)
        external
        view
        returns (uint256[] memory balances)
    {
        balances = new uint256[](tokens.length);

        for (uint256 i = 0; i < tokens.length;) {
            balances[i] = ISRC20(tokens[i]).balanceOfSigned(owner, expiry, signature);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice Batch read balances using staticcall (reverts on any failure for backwards compatibility)
    function batchBalances(address owner, address[] calldata tokens, uint256 expiry, bytes calldata signature)
        external
        view
        returns (uint256[] memory balances)
    {
        balances = new uint256[](tokens.length);

        for (uint256 i = 0; i < tokens.length;) {
            bytes memory data =
                abi.encodeWithSignature("balanceOfSigned(address,uint256,bytes)", owner, expiry, signature);

            (bool success, bytes memory returnData) = tokens[i].staticcall(data);

            if (success && returnData.length >= 32) {
                balances[i] = abi.decode(returnData, (uint256));
            } else {
                // Revert with detailed error info
                revert(string(abi.encodePacked("Failed to read balance from token at index ", _toString(i))));
            }

            unchecked {
                ++i;
            }
        }
    }

    /// @notice Convert uint to string for error messages
    function _toString(uint256 value) internal pure returns (string memory) {
        if (value == 0) return "0";

        uint256 temp = value;
        uint256 digits;
        while (temp != 0) {
            digits++;
            temp /= 10;
        }

        bytes memory buffer = new bytes(digits);
        while (value != 0) {
            digits -= 1;
            buffer[digits] = bytes1(uint8(48 + uint256(value % 10)));
            value /= 10;
        }

        return string(buffer);
    }
}
