// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

interface IDirectory {
    event KeySet(address indexed addr);

    function nonce() external view returns (uint96);

    function setKey(suint256 _key) external;

    function getKey() external view returns (uint256);

    function checkHasKey(address _addr) external view returns (bool);

    function keyHash(address to) external view returns (bytes32);

    function encrypt(address to, bytes memory _plaintext) external returns (bytes memory);

    function decrypt(bytes memory _encryptedData) external view returns (bytes memory);

    function packEncryptedData(bytes memory _ciphertext, uint96 _nonce) external pure returns (bytes memory);

    function parseEncryptedData(bytes memory _encryptedData) external pure returns (bytes memory ct, uint96 nce);
}
