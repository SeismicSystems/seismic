// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import {SRC20} from "seismic-std-lib/SRC20.sol";

/// @notice Concrete SRC20 token deployed by the SRC20Factory.
/// @dev Inherits the full SRC20 standard including encrypted events,
///      EIP-2612 permit, and signed balance reads.
contract SRC20Token is SRC20 {
    address public owner;

    constructor(string memory _name, string memory _symbol, uint8 _decimals, suint256 _initialSupply, address _owner)
        SRC20(_name, _symbol, _decimals)
    {
        owner = _owner;
        _mint(_owner, _initialSupply);
    }

    function totalSupply() public view returns (uint256) {
        return _totalSupply();
    }

    function mint(address to, suint256 amount) external {
        require(msg.sender == owner, "NOT_OWNER");
        _mint(to, amount);
    }

    function burn(address from, suint256 amount) external {
        require(msg.sender == owner, "NOT_OWNER");
        _burn(from, amount);
    }
}
