// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

/// @title CryptoUtils
/// @notice Library for interacting with Seismic crypto precompiles (RNG, AES-256-GCM, HKDF)
library CryptoUtils {
    ////////////////////////////////////////////////////////////////////////
    // Errors
    ////////////////////////////////////////////////////////////////////////

    error RNGPrecompileCallFailed();
    error AESPrecompileCallFailed();
    error EncryptionReturnedNoOutput();
    error CiphertextCannotBeEmpty();
    error HKDFPrecompileCallFailed();
    error InvalidHKDFOutputLength(uint256 returnedLength);

    ////////////////////////////////////////////////////////////////////////
    // Precompile Addresses
    ////////////////////////////////////////////////////////////////////////

    /// @dev Precompile address for random number generation
    address private constant RNG_PRECOMPILE = address(0x64);

    /// @dev Precompile address for AES encryption
    address private constant AES_ENCRYPT_PRECOMPILE = address(0x66);

    /// @dev Precompile address for AES decryption
    address private constant AES_DECRYPT_PRECOMPILE = address(0x67);

    /// @dev Precompile address for HKDF key derivation
    address private constant HKDF_PRECOMPILE = address(0x68);

    ////////////////////////////////////////////////////////////////////////
    // RNG
    ////////////////////////////////////////////////////////////////////////

    /// @notice Generates a random 96-bit nonce via the RNG precompile
    /// @return A 96-bit random nonce
    function generateRandomNonce() internal view returns (uint96) {
        (bool success, bytes memory output) = RNG_PRECOMPILE.staticcall(abi.encodePacked(uint32(32)));
        if (!success) revert RNGPrecompileCallFailed();

        bytes32 randomBytes;
        assembly {
            randomBytes := mload(add(output, 32))
        }

        return uint96(uint256(randomBytes));
    }

    /// @notice Generates a random 256-bit AES key via the RNG precompile
    /// @return A random shielded uint256 suitable for use as an AES key
    function generateRandomAESKey() internal view returns (suint256) {
        bytes memory personalization = abi.encodePacked("aes-key", block.timestamp);
        bytes memory input = abi.encodePacked(uint32(32), personalization);

        (bool success, bytes memory output) = RNG_PRECOMPILE.staticcall(input);
        if (!success) revert RNGPrecompileCallFailed();

        bytes32 randomBytes;
        assembly {
            randomBytes := mload(add(output, 32))
        }

        return suint256(randomBytes);
    }

    ////////////////////////////////////////////////////////////////////////
    // AES-256-GCM
    ////////////////////////////////////////////////////////////////////////

    /// @notice Encrypts plaintext using AES-256-GCM via the encryption precompile
    /// @param key The 32-byte AES-256-GCM key
    /// @param nonce The 96-bit nonce; caller must avoid reuse
    /// @param plaintext The data to encrypt
    /// @return ciphertext The encrypted data
    function encrypt(suint256 key, uint96 nonce, bytes memory plaintext)
        internal
        view
        returns (bytes memory ciphertext)
    {
        bytes memory input = abi.encodePacked(key, nonce, plaintext);
        (bool success, bytes memory output) = AES_ENCRYPT_PRECOMPILE.staticcall(input);
        if (!success) revert AESPrecompileCallFailed();
        if (output.length == 0) revert EncryptionReturnedNoOutput();
        return output;
    }

    /// @notice Decrypts ciphertext using AES-256-GCM via the decryption precompile
    /// @param key The 32-byte AES-256-GCM key
    /// @param nonce The 96-bit nonce used during encryption
    /// @param ciphertext The encrypted data
    /// @return plaintext The decrypted data
    function decrypt(suint256 key, uint96 nonce, bytes memory ciphertext)
        internal
        view
        returns (bytes memory plaintext)
    {
        if (ciphertext.length == 0) revert CiphertextCannotBeEmpty();
        bytes memory input = abi.encodePacked(key, nonce, ciphertext);
        (bool success, bytes memory output) = AES_DECRYPT_PRECOMPILE.staticcall(input);
        if (!success) revert AESPrecompileCallFailed();
        return output;
    }

    ////////////////////////////////////////////////////////////////////////
    // HKDF
    ////////////////////////////////////////////////////////////////////////

    /// @notice Derives a 32-byte key using HKDF via the precompile
    /// @param input Arbitrary input data (salt, info, or keying material)
    /// @return result The derived 32-byte key as a shielded uint
    function HKDFDeriveKey(bytes memory input) internal view returns (suint result) {
        (bool success, bytes memory output) = HKDF_PRECOMPILE.staticcall(input);
        if (!success) revert HKDFPrecompileCallFailed();
        if (output.length != 32) revert InvalidHKDFOutputLength(output.length);

        assembly {
            result := mload(add(output, 32))
        }
    }
}
