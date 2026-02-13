// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IDirectory} from "../directory/IDirectory.sol";
import {IIntelligence} from "./IIntelligence.sol";

contract Intelligence is IIntelligence {
    address public constant DIRECTORY_ADDRESS = address(0x1000000000000000000000000000000000000004);
    IDirectory public constant directory = IDirectory(DIRECTORY_ADDRESS);

    address public constant INITIAL_OWNER = address(0x6346d64A3f31774283b72926B75Ffda9662266ce);
    address public owner;

    address[] public providers;

    function numProviders() public view returns (uint256) {
        return providers.length;
    }

    function encryptToProviders(bytes memory _plaintext) external returns (bytes32[] memory, bytes[] memory) {
        bytes32[] memory hashes = new bytes32[](numProviders());
        bytes[] memory encryptedData = new bytes[](numProviders());
        for (uint256 i = 0; i < numProviders(); i++) {
            hashes[i] = directory.keyHash(providers[i]);
            encryptedData[i] = directory.encrypt(providers[i], _plaintext);
        }
        return (hashes, encryptedData);
    }

    function addProvider(address _provider) external uniqueProvider(_provider) onlyOwner {
        providers.push(_provider);

        emit ProviderAdded(_provider);
    }

    function removeProvider(address _provider) external onlyOwner {
        uint256 idx = findProvider(_provider);
        if (idx == type(uint256).max) {
            revert("PROVIDER_NOT_FOUND");
        }

        providers[idx] = providers[providers.length - 1];
        providers.pop();

        emit ProviderRemoved(_provider);
    }

    function findProvider(address _provider) internal view returns (uint256) {
        for (uint256 i = 0; i < numProviders(); i++) {
            if (providers[i] == _provider) {
                return i;
            }
        }
        return type(uint256).max;
    }

    function transferOwnership(address newOwner) public virtual onlyOwner {
        owner = newOwner;
        emit OwnershipTransferred(newOwner);
    }

    modifier uniqueProvider(address _provider) {
        require(findProvider(_provider) == type(uint256).max, "DUPLICATE_PROVIDER");
        _;
    }

    modifier onlyOwner() virtual {
        if (owner == address(0)) {
            owner = INITIAL_OWNER;
        }
        require(msg.sender == owner, "UNAUTHORIZED");
        _;
    }
}
