// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

/**
 * @title AesLib
 * @notice This library provides functions to interact with AES-256-GCM
 *         encryption/decryption and HKDF key derivation precompiled contracts.
 * @dev It includes three constants referencing the addresses of the precompiled
 *      contracts:
 *        > AES_ENC_PRECOMPILE (encryption),
 *        > AES_DEC_PRECOMPILE (decryption),
 *        > HKDF_PRECOMPILE (key derivation).
 */
library AesLib {
    /**
     * @dev Custom errors for more gas-efficient reverts.
     */
    error AESPrecompileCallFailed();
    error HKDFPrecompileCallFailed();
    error InvalidHKDFOutputLength(uint256 returnedLength);

    /**
     * @notice The address of the AES encryption precompiled contract.
     */
    address public constant AES_ENC_PRECOMPILE = address(0x66);

    /**
     * @notice The address of the AES decryption precompiled contract.
     */
    address public constant AES_DEC_PRECOMPILE = address(0x67);

    /**
     * @notice The address of the HKDF key derivation precompiled contract.
     */
    address public constant HKDF_PRECOMPILE = address(0x68);

    /**
     * @notice Encrypts the plaintext using AES-256-GCM with the provided key and nonce.
     *
     * @param aes_key The 32-byte AES-256-GCM key used for encryption (suint256).
     * @param nonce   The 96-bit nonce (uint96) used for encryption; Caller is
     *                responsible for avoiding repeat use. Nonce must be known by
     *                the future decryption party.
     * @param plaintext The data to encrypt.
     * @return Encrypted ciphertext as bytes.
     *
     * Requirements:
     * - The staticcall to the AES_ENC_PRECOMPILE must succeed.
     * - If the precompile call fails, this function reverts with `AESPrecompileCallFailed()`.
     */
    function AES256GCMEncrypt(suint256 aes_key, uint96 nonce, bytes memory plaintext)
        internal
        view
        returns (bytes memory)
    {
        // Concatenate and encode input args for the precompile
        bytes memory input = abi.encodePacked(aes_key, nonce, plaintext);

        // Call the AES encryption precompiled contract
        (bool success, bytes memory output) = AES_ENC_PRECOMPILE.staticcall(input);

        if (!success) {
            revert AESPrecompileCallFailed();
        }

        return output;
    }

    /**
     * @notice Decrypts the ciphertext using AES-256-GCM with the provided key and nonce.
     * @param aes_key The 32-byte AES-256-GCM key used for decryption (suint256).
     * @param nonce   The 96-bit nonce (uint96) used for encryption; Caller is responsible for avoiding repeat use.
     * @param ciphertext The data to decrypt.
     * @return Decrypted plaintext as bytes.
     *
     * Requirements:
     * - The staticcall to the AES_DEC_PRECOMPILE must succeed.
     * - If the precompile call fails, this function reverts with `AESPrecompileCallFailed()`.
     */
    function AES256GCMDecrypt(suint256 aes_key, uint96 nonce, bytes memory ciphertext)
        internal
        view
        returns (bytes memory)
    {
        // Concatenate and encode input args for the precompile
        bytes memory input = abi.encodePacked(aes_key, nonce, ciphertext);

        // Call the AES decryption precompiled contract
        (bool success, bytes memory output) = AES_DEC_PRECOMPILE.staticcall(input);

        if (!success) {
            revert AESPrecompileCallFailed();
        }

        // Return the decrypted plaintext directly
        return output;
    }

    /**
     * @notice Derives an AES key (bytes32) using HKDF via a precompiled contract.
     * @dev The function uses the HKDF precompiled contract. For a secure derived key,
     *      the caller must provide sufficiently random and high-entropy input data.
     *      The security of the result directly depends on the unpredictability of this input
     * @param input Arbitrary input data (salt, info, or keying material) to be used by HKDF.
     * @return result The derived 32-byte key as an suint.
     *
     * Requirements:
     * - The staticcall to the HKDF_PRECOMPILE must succeed.
     * - The precompile is expected to return exactly 32 bytes. Otherwise, it reverts with `InvalidHKDFOutputLength()`.
     */
    function HKDFDeriveKey(bytes memory input) internal view returns (suint result) {
        (bool success, bytes memory output) = HKDF_PRECOMPILE.staticcall(input);

        if (!success) {
            revert HKDFPrecompileCallFailed();
        }

        if (output.length != 32) {
            revert InvalidHKDFOutputLength(output.length);
        }

        assembly {
            result := mload(add(output, 32))
        }
    }

    function packEncryptedData(bytes memory _ciphertext, uint96 _nonce) public pure returns (bytes memory) {
        return abi.encodePacked(_ciphertext, _nonce);
    }

    function parseEncryptedData(bytes memory _encryptedData)
        public
        pure
        returns (bytes memory ciphertext, uint96 nonce)
    {
        uint256 nonceStart = _encryptedData.length - 12;

        ciphertext = new bytes(nonceStart);
        for (uint256 i = 0; i < nonceStart; i++) {
            ciphertext[i] = _encryptedData[i];
        }

        bytes memory nonceBytes = new bytes(12);
        for (uint256 i = 0; i < 12; i++) {
            nonceBytes[i] = _encryptedData[nonceStart + i];
        }
        nonce = uint96(bytes12(nonceBytes));
    }
}
