// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IDirectory} from "../directory/IDirectory.sol";

interface IIntelligence {
    event OwnershipTransferred(address indexed owner);
    event ProviderAdded(address indexed provider);
    event ProviderRemoved(address indexed provider);

    function DIRECTORY_ADDRESS() external view returns (address);
    function directory() external view returns (IDirectory);
    function INITIAL_OWNER() external view returns (address);
    function owner() external view returns (address);
    function providers(uint256) external view returns (address);

    function numProviders() external view returns (uint256);

    function encryptToProviders(bytes memory _plaintext) external returns (bytes32[] memory, bytes[] memory);

    function addProvider(address _provider) external;

    function removeProvider(address _provider) external;

    function transferOwnership(address newOwner) external;
}
