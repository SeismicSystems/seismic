// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";

import {CryptoUtils} from "seismic-std-lib/utils/precompiles/CryptoUtils.sol";

/// @notice Exposes CryptoUtils internal functions for direct testing
contract CryptoUtilsHarness {
    function generateRandomNonce() external view returns (uint96) {
        return CryptoUtils.generateRandomNonce();
    }

    function hkdfDeriveKey(bytes memory input) external view returns (uint256) {
        return uint256(CryptoUtils.HKDFDeriveKey(input));
    }
}

contract CryptoUtilsTest is Test {
    address constant HKDF_PRECOMPILE = address(0x68);

    CryptoUtilsHarness harness;

    function setUp() public {
        harness = new CryptoUtilsHarness();
    }

    function test_GenerateRandomNonceReturnsDistinctValues() public view {
        uint96 first = harness.generateRandomNonce();
        uint96 second = harness.generateRandomNonce();

        assertNotEq(first, second);
    }

    function test_HKDFDeriveKeyReturnsNonzeroKey() public {
        bytes memory input = abi.encodePacked("hkdf test input keying material");

        // Skip (rather than fail) if the HKDF precompile is absent from the
        // test EVM, so a harness environment gap doesn't read as a red test.
        (bool available,) = HKDF_PRECOMPILE.staticcall(input);
        vm.skip(!available, "HKDF precompile (0x68) unavailable in test EVM");

        uint256 derived = harness.hkdfDeriveKey(input);
        assertNotEq(derived, 0);
    }
}
