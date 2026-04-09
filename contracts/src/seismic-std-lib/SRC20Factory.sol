// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import {SRC20Token} from "./SRC20Token.sol";

/// @notice Factory that deploys new SRC20 token instances.
/// @dev Each createToken call deploys a full SRC20Token contract (not a clone)
///      because SRC20 uses immutable fields that must be set in the constructor.
contract SRC20Factory {
    event TokenCreated(address indexed creator, address indexed token, string name, string symbol);

    address[] public tokens;

    function createToken(string memory name, string memory symbol, uint8 decimals, suint256 initialSupply)
        external
        returns (address)
    {
        SRC20Token token = new SRC20Token(name, symbol, decimals, initialSupply, msg.sender);
        tokens.push(address(token));
        emit TokenCreated(msg.sender, address(token), name, symbol);
        return address(token);
    }

    function getTokenCount() external view returns (uint256) {
        return tokens.length;
    }
}
