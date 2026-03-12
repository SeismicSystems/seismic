// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import {MockSRC20} from "./utils/MockSRC20.sol";

contract SRC20Test is Test {
    MockSRC20 token;

    bytes32 constant PERMIT_TYPEHASH =
        keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

    function setUp() public {
        token = new MockSRC20("Token", "TKN", 18);
    }

    function invariantMetadata() public view {
        assertEq(token.name(), "Token");
        assertEq(token.symbol(), "TKN");
        assertEq(token.decimals(), 18);
    }

    function testMint() public {
        token.mint(address(0xBEEF), suint256(1e18));

        assertEq(token.totalSupply(), 1e18);
        vm.prank(address(0xBEEF));
        assertEq(token.balance(), 1e18);
    }

    function testBurn() public {
        token.mint(address(0xBEEF), suint256(1e18));
        token.burn(address(0xBEEF), suint256(0.9e18));

        assertEq(token.totalSupply(), 1e18 - 0.9e18);
        vm.prank(address(0xBEEF));
        assertEq(token.balance(), 0.1e18);
    }

    function testApprove() public {
        assertTrue(token.approve(address(0xBEEF), suint256(1e18)));

        assertEq(token.allowance(address(0xBEEF)), 1e18);
    }

    function testTransfer() public {
        token.mint(address(this), suint256(1e18));

        assertTrue(token.transfer(address(0xBEEF), suint256(1e18)));
        assertEq(token.totalSupply(), 1e18);

        assertEq(token.balance(), 0);
        vm.prank(address(0xBEEF));
        assertEq(token.balance(), 1e18);
    }

    function testTransferFrom() public {
        address from = address(0xABCD);

        token.mint(from, suint256(1e18));

        vm.prank(from);
        token.approve(address(this), suint256(1e18));

        assertTrue(token.transferFrom(from, address(0xBEEF), suint256(1e18)));
        assertEq(token.totalSupply(), 1e18);

        assertEq(token.allowance(address(this)), 0);

        assertEq(token.balance(), 0);
        vm.prank(address(0xBEEF));
        assertEq(token.balance(), 1e18);
    }

    function testInfiniteApproveTransferFrom() public {
        address from = address(0xABCD);

        token.mint(from, suint256(1e18));

        vm.prank(from);
        token.approve(address(this), suint256(type(uint256).max));

        assertTrue(token.transferFrom(from, address(0xBEEF), suint256(1e18)));
        assertEq(token.totalSupply(), 1e18);

        vm.prank(address(from));
        assertEq(token.allowance(address(this)), type(uint256).max);

        vm.prank(address(from));
        assertEq(token.balance(), 0);
        vm.prank(address(0xBEEF));
        assertEq(token.balance(), 1e18);
    }

    function testPermit() public {
        uint256 privateKey = 0xBEEF;
        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, address(0xCAFE), 1e18, 0, block.timestamp))
                )
            )
        );

        token.permit(owner, address(0xCAFE), suint256(1e18), block.timestamp, v, r, s);

        vm.prank(address(owner));
        assertEq(token.allowance(address(0xCAFE)), 1e18);
        assertEq(token.nonces(owner), 1);
    }

    function test_RevertIfTransferInsufficientBalance() public {
        token.mint(address(this), suint256(0.9e18));
        vm.expectRevert();
        token.transfer(address(0xBEEF), suint256(1e18));
    }

    function test_RevertIfTransferFromInsufficientAllowance() public {
        address from = address(0xABCD);

        token.mint(from, suint256(1e18));

        vm.prank(from);
        token.approve(address(this), suint256(0.9e18));

        vm.expectRevert();
        token.transferFrom(from, address(0xBEEF), suint256(1e18));
    }

    function test_RevertIfTransferFromInsufficientBalance() public {
        address from = address(0xABCD);

        token.mint(from, suint256(0.9e18));

        vm.prank(from);
        token.approve(address(this), suint256(1e18));

        vm.expectRevert();
        token.transferFrom(from, address(0xBEEF), suint256(1e18));
    }

    function test_RevertIfPermitBadNonce() public {
        uint256 privateKey = 0xBEEF;
        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, address(0xCAFE), 1e18, 1, block.timestamp))
                )
            )
        );

        vm.expectRevert();
        token.permit(owner, address(0xCAFE), suint256(1e18), block.timestamp, v, r, s);
    }

    function test_RevertIfPermitBadDeadline() public {
        uint256 privateKey = 0xBEEF;
        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, address(0xCAFE), 1e18, 0, block.timestamp))
                )
            )
        );

        vm.expectRevert();
        token.permit(owner, address(0xCAFE), suint256(1e18), block.timestamp + 1, v, r, s);
    }

    function test_RevertIfPermitPastDeadline() public {
        uint256 oldTimestamp = block.timestamp;
        uint256 privateKey = 0xBEEF;
        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, address(0xCAFE), 1e18, 0, oldTimestamp))
                )
            )
        );

        vm.warp(block.timestamp + 1);
        vm.expectRevert();
        token.permit(owner, address(0xCAFE), suint256(1e18), oldTimestamp, v, r, s);
    }

    function test_RevertIfPermitReplay() public {
        uint256 privateKey = 0xBEEF;
        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, address(0xCAFE), 1e18, 0, block.timestamp))
                )
            )
        );

        token.permit(owner, address(0xCAFE), suint256(1e18), block.timestamp, v, r, s);
        vm.expectRevert();
        token.permit(owner, address(0xCAFE), suint256(1e18), block.timestamp, v, r, s);
    }

    function testMetadata(string calldata name, string calldata symbol, uint8 decimals) public {
        MockSRC20 tkn = new MockSRC20(name, symbol, decimals);
        assertEq(tkn.name(), name);
        assertEq(tkn.symbol(), symbol);
        assertEq(tkn.decimals(), decimals);
    }

    function testMint(address from, uint256 amount) public {
        token.mint(from, suint256(amount));

        assertEq(token.totalSupply(), amount);
        vm.prank(address(from));
        assertEq(token.balance(), amount);
    }

    function testBurn(address from, uint256 mintAmount, uint256 burnAmount) public {
        burnAmount = bound(burnAmount, 0, mintAmount);

        token.mint(from, suint256(mintAmount));
        token.burn(from, suint256(burnAmount));

        assertEq(token.totalSupply(), mintAmount - burnAmount);
        vm.prank(address(from));
        assertEq(token.balance(), mintAmount - burnAmount);
    }

    function testApprove(address to, uint256 amount) public {
        assertTrue(token.approve(to, suint256(amount)));

        assertEq(token.allowance(to), amount);
    }

    function testTransfer(address from, uint256 amount) public {
        token.mint(address(this), suint256(amount));

        assertTrue(token.transfer(from, suint256(amount)));
        assertEq(token.totalSupply(), amount);

        if (address(this) == from) {
            assertEq(token.balance(), amount);
        } else {
            assertEq(token.balance(), 0);
            vm.prank(address(from));
            assertEq(token.balance(), amount);
        }
    }

    function testTransferFrom(address to, uint256 approval, uint256 amount) public {
        amount = bound(amount, 0, approval);

        address from = address(0xABCD);

        token.mint(from, suint256(amount));

        vm.prank(from);
        token.approve(address(this), suint256(approval));

        assertTrue(token.transferFrom(from, to, suint256(amount)));
        assertEq(token.totalSupply(), amount);

        uint256 app = from == address(this) || approval == type(uint256).max ? approval : approval - amount;
        vm.prank(address(from));
        assertEq(token.allowance(address(this)), app);

        if (from == to) {
            vm.prank(address(from));
            assertEq(token.balance(), amount);
        } else {
            vm.prank(address(from));
            assertEq(token.balance(), 0);
            vm.prank(address(to));
            assertEq(token.balance(), amount);
        }
    }

    function testPermit(uint248 privKey, address to, uint256 amount, uint256 deadline) public {
        uint256 privateKey = privKey;
        if (deadline < block.timestamp) deadline = block.timestamp;
        if (privateKey == 0) privateKey = 1;

        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, to, amount, 0, deadline))
                )
            )
        );

        token.permit(owner, to, suint256(amount), deadline, v, r, s);

        vm.prank(address(owner));
        assertEq(token.allowance(to), amount);
        assertEq(token.nonces(owner), 1);
    }

    function test_RevertIfBurnInsufficientBalance(address to, uint256 mintAmount, uint256 burnAmount) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        burnAmount = bound(burnAmount, mintAmount + 1, type(uint256).max);

        token.mint(to, suint256(mintAmount));
        vm.expectRevert();
        token.burn(to, suint256(burnAmount));
    }

    function test_RevertIfTransferInsufficientBalance(address to, uint256 mintAmount, uint256 sendAmount) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        sendAmount = bound(sendAmount, mintAmount + 1, type(uint256).max);

        token.mint(address(this), suint256(mintAmount));
        vm.expectRevert();
        token.transfer(to, suint256(sendAmount));
    }

    function test_RevertIfTransferFromInsufficientAllowance(address to, uint256 approval, uint256 amount) public {
        approval = bound(approval, 0, type(uint256).max - 1);
        amount = bound(amount, approval + 1, type(uint256).max);

        address from = address(0xABCD);

        token.mint(from, suint256(amount));

        vm.prank(from);
        token.approve(address(this), suint256(approval));

        vm.expectRevert();
        token.transferFrom(from, to, suint256(amount));
    }

    function test_RevertIfTransferFromInsufficientBalance(address to, uint256 mintAmount, uint256 sendAmount) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        sendAmount = bound(sendAmount, mintAmount + 1, type(uint256).max);

        address from = address(0xABCD);

        token.mint(from, suint256(mintAmount));

        vm.prank(from);
        token.approve(address(this), suint256(sendAmount));

        vm.expectRevert();
        token.transferFrom(from, to, suint256(sendAmount));
    }

    function test_RevertIfPermitBadNonce(
        uint256 privateKey,
        address to,
        uint256 amount,
        uint256 deadline,
        uint256 nonce
    ) public {
        privateKey = boundPrivateKey(privateKey);
        if (deadline < block.timestamp) deadline = block.timestamp;
        if (nonce == 0) nonce = 1;

        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, to, amount, nonce, deadline))
                )
            )
        );

        vm.expectRevert();
        token.permit(owner, to, suint256(amount), deadline, v, r, s);
    }

    function test_RevertIfPermitBadDeadline(uint256 privateKey, address to, uint256 amount, uint256 deadline) public {
        privateKey = boundPrivateKey(privateKey);
        deadline = bound(deadline, block.timestamp, type(uint256).max - 1);

        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, to, amount, 0, deadline))
                )
            )
        );

        vm.expectRevert();
        token.permit(owner, to, suint256(amount), deadline + 1, v, r, s);
    }

    function test_RevertIfPermitPastDeadline(uint256 privateKey, address to, uint256 amount, uint256 deadline) public {
        // Ensure block.timestamp > 0 to avoid underflow
        if (block.timestamp == 0) {
            vm.warp(1);
        }
        deadline = bound(deadline, 0, block.timestamp - 1);
        privateKey = boundPrivateKey(privateKey);

        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, to, amount, 0, deadline))
                )
            )
        );

        vm.expectRevert();
        token.permit(owner, to, suint256(amount), deadline, v, r, s);
    }

    function test_RevertIfPermitReplay(uint256 privateKey, address to, uint256 amount, uint256 deadline) public {
        privateKey = boundPrivateKey(privateKey);
        if (deadline < block.timestamp) deadline = block.timestamp;

        address owner = vm.addr(privateKey);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            privateKey,
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    token.DOMAIN_SEPARATOR(),
                    keccak256(abi.encode(PERMIT_TYPEHASH, owner, to, amount, 0, deadline))
                )
            )
        );

        token.permit(owner, to, suint256(amount), deadline, v, r, s);
        vm.expectRevert();
        token.permit(owner, to, suint256(amount), deadline, v, r, s);
    }
}

contract ERC20Invariants is Test {
    BalanceSum balanceSum;
    MockSRC20 token;

    function setUp() public {
        token = new MockSRC20("Token", "TKN", 18);
        balanceSum = new BalanceSum(token);

        targetContract(address(balanceSum));
    }

    function invariantBalanceSum() public view {
        assertEq(token.totalSupply(), balanceSum.sum());
    }
}

contract BalanceSum {
    MockSRC20 token;
    uint256 public sum;

    constructor(MockSRC20 _token) {
        token = _token;
    }

    function mint(address from, uint256 amount) public {
        token.mint(from, suint256(amount));
        sum += amount;
    }

    function burn(address from, uint256 amount) public {
        token.burn(from, suint256(amount));
        sum -= amount;
    }

    function approve(address to, uint256 amount) public {
        token.approve(to, suint256(amount));
    }

    function transferFrom(address from, address to, uint256 amount) public {
        token.transferFrom(from, to, suint256(amount));
    }

    function transfer(address to, uint256 amount) public {
        token.transfer(to, suint256(amount));
    }
}
