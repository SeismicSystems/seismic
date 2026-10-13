// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ShieldedDelegationAccount} from "seismic-std-lib/ShieldedDelegationAccount.sol";
import {IShieldedDelegationAccount} from "seismic-std-lib/interfaces/IShieldedDelegationAccount.sol";
import "solady/utils/SignatureCheckerLib.sol";
import "solady/utils/P256.sol";
import "solady/utils/WebAuthn.sol";

/// @title SDAWithERC1271
/// @notice EXPERIMENT ONLY — do not ship. Models the "all keys answer ERC-1271" policy
///         under evaluation for seismic-std-lib 0.3.0, so the transitive-trust flow it
///         enables can be exercised in a test.
/// @dev Deliberately mirrors the accept-any-authorized-key semantics: every key in the
///      account's key set is consulted, with no inspection of what the digest commits to.
///      A bare bytes32 carries no domain, so the account cannot tell an Execute digest for
///      its OWN account from one belonging to a different account that merely trusts it.
contract SDAWithERC1271 is ShieldedDelegationAccount {
    bytes4 internal constant MAGIC = 0x1626ba7e;

    function isValidSignature(bytes32 digest, bytes calldata signature) external view returns (bytes4) {
        // Root-key path. Under 7702 address(this) IS the delegating EOA, so a raw ecrecover
        // against it authenticates the account's own key. Uses the precompile directly rather
        // than SignatureCheckerLib: the library would see code at address(this) and re-enter
        // this very function, recursing until the call depth limit.
        if (signature.length == 65) {
            bytes32 r;
            bytes32 s;
            uint8 v;
            assembly {
                r := calldataload(signature.offset)
                s := calldataload(add(signature.offset, 0x20))
                v := byte(0, calldataload(add(signature.offset, 0x40)))
            }
            address recovered = ecrecover(digest, v, r, s);
            if (recovered != address(0) && recovered == address(this)) return MAGIC;
        }

        Key[] storage keys = _expStorage().keys;
        uint256 len = uint256(keys.length);
        for (uint256 i = 0; i < len; i++) {
            Key storage k = keys[i];
            if (k.expiry != 0 && k.expiry <= block.timestamp) continue;
            if (_verifyExperimental(k.keyType, k.publicKey, digest, signature)) {
                return MAGIC;
            }
        }
        return 0xffffffff;
    }

    /// @dev Mirrors the parent's private `_getStorage()` slot derivation.
    struct ExpStorage {
        suint256 aesKey;
        bool aesKeyInitialized;
        Key[] keys;
        mapping(bytes32 => uint32) keyToSessionIndex;
    }

    function _expStorage() internal pure returns (ExpStorage storage $) {
        uint256 s = uint72(bytes9(keccak256("SHIELDED_DELEGATION_STORAGE")));
        assembly ("memory-safe") {
            $.slot := s
        }
    }

    /// @dev Copy of the parent's private verification logic; the parent's is `internal` but
    ///      we re-declare to keep this experiment self-contained and obviously non-shipping.
    function _verifyExperimental(KeyType keyType, bytes memory publicKey, bytes32 digest, bytes calldata signature)
        internal
        view
        returns (bool isValid)
    {
        if (keyType == KeyType.P256) {
            (bytes32 r, bytes32 s) = P256.tryDecodePointCalldata(signature);
            (bytes32 x, bytes32 y) = P256.tryDecodePoint(publicKey);
            isValid = P256.verifySignature(digest, r, s, x, y);
        } else if (keyType == KeyType.WebAuthnP256) {
            (bytes32 x, bytes32 y) = P256.tryDecodePoint(publicKey);
            isValid = WebAuthn.verify(abi.encode(digest), false, WebAuthn.tryDecodeAuth(signature), x, y);
        } else if (keyType == KeyType.Secp256k1) {
            isValid =
                SignatureCheckerLib.isValidSignatureNowCalldata(abi.decode(publicKey, (address)), digest, signature);
        }
    }
}
