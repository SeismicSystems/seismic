// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IDirectory} from "seismic-std-lib/interfaces/IDirectory.sol";
import {IIntelligence} from "seismic-std-lib/interfaces/IIntelligence.sol";

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
        uint256 keyed = 0;
        for (uint256 i = 0; i < numProviders(); i++) {
            if (directory.checkHasKey(providers[i])) {
                keyed++;
            }
        }

        bytes32[] memory hashes = new bytes32[](keyed);
        bytes[] memory encryptedData = new bytes[](keyed);
        uint256 j = 0;
        for (uint256 i = 0; i < numProviders(); i++) {
            // Never encrypt under a zero key: skip providers without a registered key.
            if (!directory.checkHasKey(providers[i])) {
                continue;
            }
            hashes[j] = directory.keyHash(providers[i]);
            encryptedData[j] = directory.encrypt(providers[i], _plaintext);
            j++;
        }
        return (hashes, encryptedData);
    }

    function addProvider(address _provider) external uniqueProvider(_provider) onlyOwner {
        require(directory.checkHasKey(_provider), "PROVIDER_NO_KEY");

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
        require(newOwner != address(0), "ZERO_OWNER");
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
