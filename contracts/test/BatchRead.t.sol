// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {SRC20} from "seismic-std-lib/SRC20.sol";
import {SRC20Multicall} from "seismic-std-lib/SRC20Multicall.sol";

contract TestSRC20 is SRC20 {
    constructor(string memory _name, string memory _symbol, uint8 _decimals) SRC20(_name, _symbol, _decimals) {}

    function mint(address to, uint256 amount) public {
        _mint(to, suint256(amount));
    }

    function totalSupply() public view returns (uint256) {
        return _totalSupply();
    }
}

/// @notice Answers balanceOfSigned without reverting but returns fewer than 32 bytes.
contract ShortReturnToken {
    fallback() external {
        assembly {
            return(0, 16)
        }
    }
}

/// @notice Answers balanceOfSigned without reverting but returns more than 32 bytes.
contract LongReturnToken {
    fallback() external {
        assembly {
            mstore(0, 42)
            return(0, 48)
        }
    }
}

/// @notice Reverts on any call.
contract RevertingToken {
    fallback() external {
        revert("RevertingToken: always reverts");
    }
}

contract BatchReadTest is Test {
    SRC20Multicall multicall;
    TestSRC20[] tokens;

    uint256 constant USER_PRIVATE_KEY = 0xBEEF;
    address user;

    uint256 constant NUM_TOKENS = 10;

    function setUp() public {
        user = vm.addr(USER_PRIVATE_KEY);

        multicall = new SRC20Multicall();

        // Deploy 10 tokens with different balances
        for (uint256 i = 0; i < NUM_TOKENS; i++) {
            TestSRC20 token = new TestSRC20("Token", "TKN", 18);
            token.mint(user, (i + 1) * 1e18);
            tokens.push(token);
        }
    }

    function _tokenAddresses() internal view returns (address[] memory addrs) {
        addrs = new address[](NUM_TOKENS);
        for (uint256 i = 0; i < NUM_TOKENS; i++) {
            addrs[i] = address(tokens[i]);
        }
    }

    function _signBalanceRead(uint256 privateKey, address owner, uint256 expiry) internal returns (bytes memory) {
        bytes32 messageHash = keccak256(abi.encodePacked("SRC20_BALANCE_READ", owner, expiry));
        bytes32 ethSignedHash = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, ethSignedHash);
        return abi.encodePacked(r, s, v);
    }

    function test_singleSignedRead() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        uint256 bal = tokens[0].balanceOfSigned(user, expiry, sig);
        assertEq(bal, 1e18);
    }

    function test_batchReadTenTokens() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        uint256[] memory balances = multicall.batchBalances(user, _tokenAddresses(), expiry, sig);

        assertEq(balances.length, NUM_TOKENS);
        for (uint256 i = 0; i < NUM_TOKENS; i++) {
            assertEq(balances[i], (i + 1) * 1e18);
        }
    }

    function test_revertExpiredSignature() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        // Warp past expiry
        vm.warp(expiry + 1);

        vm.expectRevert("signature expired");
        tokens[0].balanceOfSigned(user, expiry, sig);
    }

    function test_revertInvalidSignature() public {
        uint256 expiry = block.timestamp + 1 hours;
        uint256 wrongKey = 0xDEAD;
        bytes memory sig = _signBalanceRead(wrongKey, user, expiry);

        vm.expectRevert("invalid signature");
        tokens[0].balanceOfSigned(user, expiry, sig);
    }

    function test_selfBalanceRead() public {
        vm.prank(user);
        assertEq(tokens[0].balance(), 1e18);
    }

    function test_signatureReuse() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        // Same sig works across different tokens
        assertEq(tokens[0].balanceOfSigned(user, expiry, sig), 1e18);
        assertEq(tokens[4].balanceOfSigned(user, expiry, sig), 5e18);
        assertEq(tokens[9].balanceOfSigned(user, expiry, sig), 10e18);
    }

    // timestamp == expiry is valid
    function test_expiryBoundary() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        // Warp to exactly expiry - should still work
        vm.warp(expiry);
        uint256 bal = tokens[0].balanceOfSigned(user, expiry, sig);
        assertEq(bal, 1e18);

        // Warp 1 second past - should fail
        vm.warp(expiry + 1);
        vm.expectRevert("signature expired");
        tokens[0].balanceOfSigned(user, expiry, sig);
    }

    function test_batchMethodsConsistency() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        // Get results from both batch methods
        uint256[] memory interfaceResults = multicall.batchBalancesInterface(user, _tokenAddresses(), expiry, sig);
        uint256[] memory staticcallResults = multicall.batchBalances(user, _tokenAddresses(), expiry, sig);

        // Should have same length
        assertEq(interfaceResults.length, staticcallResults.length);
        assertEq(interfaceResults.length, NUM_TOKENS);

        // All balances should match
        for (uint256 i = 0; i < NUM_TOKENS; i++) {
            assertEq(interfaceResults[i], staticcallResults[i]);
            assertEq(interfaceResults[i], (i + 1) * 1e18);
        }
    }

    function test_batchBalancesDetailed() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        SRC20Multicall.BalanceResult[] memory results =
            multicall.batchBalancesDetailed(user, _tokenAddresses(), expiry, sig);

        assertEq(results.length, NUM_TOKENS);

        for (uint256 i = 0; i < NUM_TOKENS; i++) {
            assertTrue(results[i].success);
            assertEq(results[i].token, address(tokens[i]));
            assertEq(results[i].balance, (i + 1) * 1e18);
        }
    }

    // Zellic 3.6: a non-reverting token that returns a payload other than 32 bytes
    // must be reported as a failure, not as a successful zero balance.
    function test_batchBalancesDetailedMalformedReturn() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        address[] memory testTokens = new address[](4);
        testTokens[0] = address(tokens[0]);
        testTokens[1] = address(new ShortReturnToken());
        testTokens[2] = address(new LongReturnToken());
        testTokens[3] = address(new RevertingToken());

        SRC20Multicall.BalanceResult[] memory results = multicall.batchBalancesDetailed(user, testTokens, expiry, sig);

        assertEq(results.length, 4);

        // (a) well-formed token: success with the real balance
        assertTrue(results[0].success);
        assertEq(results[0].balance, 1e18);

        // (b) malformed returns (16 and 48 bytes): must not look like a real zero balance
        assertFalse(results[1].success);
        assertEq(results[1].balance, 0);
        assertFalse(results[2].success);
        assertEq(results[2].balance, 0);

        // (c) reverting token: failure
        assertFalse(results[3].success);
        assertEq(results[3].balance, 0);
    }

    function test_staticcallErrorHandling() public {
        uint256 expiry = block.timestamp + 1 hours;
        bytes memory sig = _signBalanceRead(USER_PRIVATE_KEY, user, expiry);

        // Create array with expired signature for second call
        address[] memory testTokens = new address[](2);
        testTokens[0] = address(tokens[0]);
        testTokens[1] = address(tokens[1]);

        // Test with expired signature
        vm.warp(expiry + 1); // Move past expiry

        // Detailed version should not revert, but show failure
        SRC20Multicall.BalanceResult[] memory results = multicall.batchBalancesDetailed(user, testTokens, expiry, sig);

        assertEq(results.length, 2);
        assertFalse(results[0].success); // Should fail due to expired signature
        assertFalse(results[1].success); // Should fail due to expired signature
        assertEq(results[0].balance, 0);
        assertEq(results[1].balance, 0);

        vm.expectRevert();
        multicall.batchBalances(user, testTokens, expiry, sig);
    }
}
