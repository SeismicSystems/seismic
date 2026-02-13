// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {AesLib} from "../../lib/AesLib.sol";

import {IDirectory} from "./IDirectory.sol";

contract Directory is IDirectory {
    mapping(address => suint256) private keys;
    uint96 public nonce;

    function setKey(suint256 _key) public {
        keys[msg.sender] = _key;

        emit KeySet(msg.sender);
    }

    function getKey() public view returns (uint256) {
        return uint256(keys[msg.sender]);
    }

    function checkHasKey(address _addr) public view returns (bool) {
        return keys[_addr] != suint256(0);
    }

    function keyHash(address to) public view returns (bytes32) {
        return keccak256(abi.encodePacked(keys[to]));
    }

    function encrypt(address to, bytes memory _plaintext) public returns (bytes memory) {
        suint256 key = keys[to];

        bytes memory ciphertext = AesLib.AES256GCMEncrypt(key, nonce, _plaintext);
        bytes memory encryptedData = packEncryptedData(ciphertext, nonce);

        nonce++;
        return encryptedData;
    }

    function decrypt(bytes memory _encryptedData) public view returns (bytes memory) {
        (bytes memory ct, uint96 nce) = parseEncryptedData(_encryptedData);
        return AesLib.AES256GCMDecrypt(keys[msg.sender], nce, ct);
    }

    function packEncryptedData(bytes memory _ciphertext, uint96 _nonce) public pure returns (bytes memory) {
        return abi.encodePacked(_ciphertext, _nonce);
    }

    function parseEncryptedData(bytes memory _encryptedData) public pure returns (bytes memory ct, uint96 nce) {
        uint256 nonceStart = _encryptedData.length - 12;

        ct = new bytes(nonceStart);
        for (uint256 i = 0; i < nonceStart; i++) {
            ct[i] = _encryptedData[i];
        }

        bytes memory nonceBytes = new bytes(12);
        for (uint256 i = 0; i < 12; i++) {
            nonceBytes[i] = _encryptedData[nonceStart + i];
        }
        nce = uint96(bytes12(nonceBytes));
    }
}
