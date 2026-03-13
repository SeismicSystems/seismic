// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

/// @title IShieldedDelegationAccount
/// @notice Interface for ShieldedDelegationAccount functionality
/// @dev Defines the core session management and execution functions
interface IShieldedDelegationAccount {
    /// @notice The type of key
    enum KeyType {
        P256,
        WebAuthnP256,
        Secp256k1
    }

    /// @dev A key that can be used to authorize calls.
    /// @dev spendLimit and spentWei use suint256 (shielded) in storage to hide authorization scope and spending activity.
    struct Key {
        /// @dev Unix timestamp at which the key expires (0 = never).
        uint40 expiry;
        /// @dev Type of key. See the {KeyType} enum.
        KeyType keyType;
        /// @dev Public key in encoded form.
        bytes publicKey;
        /// @dev The spend limit for the key in wei (0 = no spending, type(uint256).max = unlimited).
        suint256 spendLimit;
        /// @dev The amount of wei spent from the key.
        suint256 spentWei;
        /// @dev The nonce for the key.
        uint256 nonce;
    }

    /// @dev Public view of a key with unshielded fields, returned by getKey().
    struct KeyView {
        uint40 expiry;
        KeyType keyType;
        bytes publicKey;
        uint256 spendLimit;
        uint256 spentWei;
        uint256 nonce;
    }

    /// @notice Emitted when a new key is authorized
    /// @param keyHash The hash of the key
    /// @param keyType The type of key
    /// @param expiry The expiry timestamp
    event KeyAuthorized(bytes32 keyHash, KeyType keyType, uint40 expiry);

    /// @notice Emitted when a key is revoked
    /// @param keyHash The hash of the key
    event KeyRevoked(bytes32 keyHash);

    /// @notice Creates a new authorized session
    /// @param keyType The type of key
    /// @param publicKey The public key
    /// @param expiry The timestamp when the session expires (0 = unlimited)
    /// @param limitWei The maximum amount of wei that can be spent (0 = no spending, type(uint256).max = unlimited)
    /// @return idx The index of the newly created session
    function authorizeKey(KeyType keyType, bytes calldata publicKey, uint40 expiry, uint256 limitWei)
        external
        returns (uint32 idx);

    /// @notice Revokes an existing session
    /// @param keyType The type of key
    /// @param publicKey The public key
    function revokeKey(KeyType keyType, bytes calldata publicKey) external;

    /// @notice Sets the AES encryption key using the RNG precompile
    function setAESKey() external;

    /// @notice Gas-free helper to encrypt plaintext
    /// @param plaintext The data to encrypt
    /// @return nonce The random nonce used for encryption
    /// @return ciphertext The encrypted data
    function encrypt(bytes calldata plaintext) external view returns (uint96 nonce, bytes memory ciphertext);

    /// @notice Executes transactions after verifying session signature and decrypting payload
    /// @param nonce The nonce used for encryption/decryption
    /// @param ciphertext The encrypted transaction data
    /// @param sig The session signature authorizing the execution
    /// @param idx The index of the session to use
    function execute(uint96 nonce, bytes calldata ciphertext, bytes calldata sig, uint32 idx) external;

    /// @notice Gets the index of a key
    /// @param keyType The type of key
    /// @param publicKey The public key
    /// @return The index of the key
    function getKeyIndex(KeyType keyType, bytes calldata publicKey) external view returns (uint32);

    /// @notice Gets the current nonce for a session
    /// @param idx The index of the key
    /// @return The current nonce value
    function getKeyNonce(uint32 idx) external view returns (uint256);

    /// @notice Accessor for keys array (returns unshielded view). Only callable by the account itself.
    /// @param idx The index of the key to access
    /// @return key The key with unshielded spend fields
    function getKey(uint32 idx) external view returns (KeyView memory key);

    /// @notice Allows EOA to still receive ETH
    receive() external payable;
}
