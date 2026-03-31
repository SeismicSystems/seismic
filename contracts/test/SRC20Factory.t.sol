// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {SRC20Factory} from "seismic-std-lib/SRC20Factory.sol";
import {SRC20Token} from "seismic-std-lib/SRC20Token.sol";

contract SRC20FactoryTest is Test {
    SRC20Factory factory;

    address alice = address(0xA11CE);
    address bob = address(0xB0B);

    function setUp() public {
        factory = new SRC20Factory();
    }

    function testCreateToken() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken(
            "Test Token", "TST", 18, suint256(1_000_000e18)
        );

        SRC20Token token = SRC20Token(tokenAddr);

        assertEq(token.name(), "Test Token");
        assertEq(token.symbol(), "TST");
        assertEq(token.decimals(), 18);
        assertEq(token.owner(), alice);
        assertEq(token.totalSupply(), 1_000_000e18);

        // Owner can read their own balance
        vm.prank(alice);
        assertEq(token.balance(), 1_000_000e18);

        // Factory tracks deployed tokens
        assertEq(factory.getTokenCount(), 1);
        assertEq(factory.tokens(0), tokenAddr);
    }

    function testCreateMultipleTokens() public {
        vm.prank(alice);
        address token1 = factory.createToken("Token A", "TKA", 18, suint256(100e18));

        vm.prank(bob);
        address token2 = factory.createToken("Token B", "TKB", 8, suint256(500e8));

        assertEq(factory.getTokenCount(), 2);
        assertTrue(token1 != token2);

        assertEq(SRC20Token(token1).name(), "Token A");
        assertEq(SRC20Token(token2).name(), "Token B");
        assertEq(SRC20Token(token2).decimals(), 8);
    }

    function testTokenCreatedEvent() public {
        vm.prank(alice);
        vm.recordLogs();
        factory.createToken("Event Token", "EVT", 18, suint256(1e18));

        Vm.Log[] memory logs = vm.getRecordedLogs();

        // Find TokenCreated event (last factory event before SRC20 Transfer events)
        bool found = false;
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == keccak256("TokenCreated(address,address,string,string)")) {
                assertEq(address(uint160(uint256(logs[i].topics[1]))), alice);
                found = true;
                break;
            }
        }
        assertTrue(found, "TokenCreated event not emitted");
    }

    function testOwnerCanMint() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken("Mint Test", "MNT", 18, suint256(100e18));
        SRC20Token token = SRC20Token(tokenAddr);

        vm.prank(alice);
        token.mint(bob, suint256(50e18));

        assertEq(token.totalSupply(), 150e18);
        vm.prank(bob);
        assertEq(token.balance(), 50e18);
    }

    function testOwnerCanBurn() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken("Burn Test", "BRN", 18, suint256(100e18));
        SRC20Token token = SRC20Token(tokenAddr);

        vm.prank(alice);
        token.burn(alice, suint256(30e18));

        assertEq(token.totalSupply(), 70e18);
        vm.prank(alice);
        assertEq(token.balance(), 70e18);
    }

    function testNonOwnerCannotMint() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken("No Mint", "NOM", 18, suint256(100e18));
        SRC20Token token = SRC20Token(tokenAddr);

        vm.prank(bob);
        vm.expectRevert("NOT_OWNER");
        token.mint(bob, suint256(50e18));
    }

    function testNonOwnerCannotBurn() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken("No Burn", "NOB", 18, suint256(100e18));
        SRC20Token token = SRC20Token(tokenAddr);

        vm.prank(bob);
        vm.expectRevert("NOT_OWNER");
        token.burn(alice, suint256(50e18));
    }

    function testTransfer() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken("Transfer", "XFR", 18, suint256(100e18));
        SRC20Token token = SRC20Token(tokenAddr);

        vm.prank(alice);
        token.transfer(bob, suint256(40e18));

        vm.prank(alice);
        assertEq(token.balance(), 60e18);
        vm.prank(bob);
        assertEq(token.balance(), 40e18);
    }

    function testApproveAndTransferFrom() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken("Approval", "APR", 18, suint256(100e18));
        SRC20Token token = SRC20Token(tokenAddr);

        vm.prank(alice);
        token.approve(bob, suint256(50e18));

        vm.prank(alice);
        assertEq(token.allowance(bob), 50e18);

        vm.prank(bob);
        token.transferFrom(alice, bob, suint256(30e18));

        vm.prank(alice);
        assertEq(token.balance(), 70e18);
        vm.prank(bob);
        assertEq(token.balance(), 30e18);
    }

    function testZeroInitialSupply() public {
        vm.prank(alice);
        address tokenAddr = factory.createToken("Zero Supply", "ZRO", 18, suint256(0));
        SRC20Token token = SRC20Token(tokenAddr);

        assertEq(token.totalSupply(), 0);
        vm.prank(alice);
        assertEq(token.balance(), 0);
    }

    function testFuzzCreateToken(
        string memory name,
        string memory symbol,
        uint8 decimals,
        uint128 supply
    ) public {
        vm.prank(alice);
        address tokenAddr = factory.createToken(name, symbol, decimals, suint256(uint256(supply)));
        SRC20Token token = SRC20Token(tokenAddr);

        assertEq(token.name(), name);
        assertEq(token.symbol(), symbol);
        assertEq(token.decimals(), decimals);
        assertEq(token.owner(), alice);
        assertEq(token.totalSupply(), uint256(supply));
    }
}
