// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "seismic-std-lib/SRC20.sol";

contract WSRC20 is SRC20 {
    constructor() SRC20("Wrapped Seismic Token", "WSIZE", 18) {}

    function deposit() public payable {
        _mint(msg.sender, suint256(msg.value));
    }

    function withdraw(suint256 amount) public {
        _burn(msg.sender, amount);
        (bool success,) = msg.sender.call{value: uint256(amount)}("");
        require(success, "WSRC20: ETH transfer failed");
    }

    receive() external payable {
        deposit();
    }

    fallback() external payable {
        deposit();
    }
}
