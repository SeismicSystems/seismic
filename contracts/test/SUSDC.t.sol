// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import {SUSDC} from "../src/examples/SUSDC.sol";
import {
    TransparentUpgradeableProxy,
    ITransparentUpgradeableProxy
} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";

/// @dev Standard ERC20 interface as an Ethereum-native contract would see it.
///      Calling through this interface exercises the canonical selectors
///      (0xa9059cbb, 0x23b872dd, 0x095ea7b3, 0x70a08231, 0xdd62ed3e).
interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function approve(address spender, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
    function allowance(address owner, address spender) external view returns (uint256);
    function totalSupply() external view returns (uint256);
}

contract SUSDCTest is Test {
    SUSDC token;
    IERC20 erc20;

    address constant INTELLIGENCE_ADDR = 0x1000000000000000000000000000000000000005;
    address constant DIRECTORY_ADDR = 0x1000000000000000000000000000000000000004;

    address alice = makeAddr("alice");
    address bob = makeAddr("bob");
    address spender = makeAddr("spender");

    function setUp() public {
        deployCodeTo("Directory.sol", DIRECTORY_ADDR);
        deployCodeTo("Intelligence.sol", INTELLIGENCE_ADDR);

        token = new SUSDC();
        erc20 = IERC20(address(token));
    }

    /*//////////////////////////////////////////////////////////////
                        ERC20 COMPATIBILITY
    //////////////////////////////////////////////////////////////*/

    function test_ERC20Transfer() public {
        token.mint(alice, suint256(100e6));

        vm.prank(alice);
        assertTrue(erc20.transfer(bob, 40e6));

        vm.prank(alice);
        assertEq(erc20.balanceOf(alice), 60e6);
        vm.prank(bob);
        assertEq(erc20.balanceOf(bob), 40e6);
    }

    function test_ERC20ApproveAndTransferFrom() public {
        token.mint(alice, suint256(100e6));

        vm.prank(alice);
        assertTrue(erc20.approve(spender, 50e6));

        vm.prank(spender);
        assertTrue(erc20.transferFrom(alice, bob, 30e6));

        vm.prank(spender);
        assertEq(erc20.allowance(alice, spender), 20e6);
        vm.prank(bob);
        assertEq(erc20.balanceOf(bob), 30e6);
    }

    function test_ERC20TransferFromInfiniteAllowanceNotDecremented() public {
        token.mint(alice, suint256(100e6));

        vm.prank(alice);
        erc20.approve(spender, type(uint256).max);

        vm.prank(spender);
        erc20.transferFrom(alice, bob, 30e6);

        vm.prank(spender);
        assertEq(erc20.allowance(alice, spender), type(uint256).max);
    }

    function test_RevertWhen_ERC20TransferFromExceedsAllowance() public {
        token.mint(alice, suint256(100e6));

        vm.prank(alice);
        erc20.approve(spender, 10e6);

        vm.prank(spender);
        vm.expectRevert();
        erc20.transferFrom(alice, bob, 20e6);
    }

    /*//////////////////////////////////////////////////////////////
                        GATED VIEW FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    function test_RevertWhen_BalanceOfCalledByOtherAccount() public {
        token.mint(alice, suint256(100e6));

        vm.prank(bob);
        vm.expectRevert("SUSDC: caller is not account");
        erc20.balanceOf(alice);
    }

    function test_AllowanceReadableByOwnerAndSpender() public {
        vm.prank(alice);
        erc20.approve(spender, 50e6);

        vm.prank(alice);
        assertEq(erc20.allowance(alice, spender), 50e6);
        vm.prank(spender);
        assertEq(erc20.allowance(alice, spender), 50e6);
    }

    function test_RevertWhen_AllowanceCalledByThirdParty() public {
        vm.prank(alice);
        erc20.approve(spender, 50e6);

        vm.prank(bob);
        vm.expectRevert("SUSDC: caller is not owner or spender");
        erc20.allowance(alice, spender);
    }

    /*//////////////////////////////////////////////////////////////
                        PAUSE ENFORCEMENT
    //////////////////////////////////////////////////////////////*/

    function test_RevertWhen_ERC20TransferWhilePaused() public {
        token.mint(alice, suint256(100e6));
        token.pause();

        vm.prank(alice);
        vm.expectRevert("SUSDC: paused");
        erc20.transfer(bob, 10e6);
    }

    function test_RevertWhen_ERC20TransferFromWhilePaused() public {
        token.mint(alice, suint256(100e6));
        vm.prank(alice);
        erc20.approve(spender, 50e6);

        token.pause();

        vm.prank(spender);
        vm.expectRevert("SUSDC: paused");
        erc20.transferFrom(alice, bob, 10e6);
    }

    function test_RevertWhen_ERC20ApproveWhilePaused() public {
        token.pause();

        vm.prank(alice);
        vm.expectRevert("SUSDC: paused");
        erc20.approve(spender, 50e6);
    }

    /*//////////////////////////////////////////////////////////////
                    SHIELDED AND PUBLIC PATHS SHARE STATE
    //////////////////////////////////////////////////////////////*/

    function test_ShieldedApproveSpendableViaERC20TransferFrom() public {
        token.mint(alice, suint256(100e6));

        vm.prank(alice);
        token.approve(spender, suint256(50e6));

        vm.prank(spender);
        assertTrue(erc20.transferFrom(alice, bob, 50e6));

        vm.prank(bob);
        assertEq(erc20.balanceOf(bob), 50e6);
    }
}

contract SUSDCProxyTest is Test {
    // keccak256("eip1967.proxy.admin") - 1
    bytes32 constant ADMIN_SLOT = 0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103;

    address constant INTELLIGENCE_ADDR = 0x1000000000000000000000000000000000000005;
    address constant DIRECTORY_ADDR = 0x1000000000000000000000000000000000000004;

    SUSDC implementation;
    SUSDC token; // the proxy, viewed through the SUSDC ABI
    ProxyAdmin proxyAdmin;

    address admin = makeAddr("admin");
    address proxyOwner = makeAddr("proxyOwner");
    address alice = makeAddr("alice");
    address bob = makeAddr("bob");

    function setUp() public {
        deployCodeTo("Directory.sol", DIRECTORY_ADDR);
        deployCodeTo("Intelligence.sol", INTELLIGENCE_ADDR);

        implementation = new SUSDC();
        TransparentUpgradeableProxy proxy = new TransparentUpgradeableProxy(
            address(implementation), proxyOwner, abi.encodeCall(SUSDC.initialize, (admin))
        );
        token = SUSDC(address(proxy));
        proxyAdmin = ProxyAdmin(address(uint160(uint256(vm.load(address(proxy), ADMIN_SLOT)))));
    }

    function test_InitializeSetsStateInProxyStorage() public view {
        assertEq(token.admin(), admin);
        assertEq(token.name(), "Shielded USD Coin");
        assertEq(token.symbol(), "SUSDC");
        assertEq(token.decimals(), 6);
    }

    function test_RevertWhen_InitializeCalledTwice() public {
        vm.expectRevert();
        token.initialize(alice);
    }

    function test_RevertWhen_ImplementationInitializedDirectly() public {
        vm.expectRevert();
        implementation.initialize(alice);
    }

    function test_ShieldedBalancesLiveInProxyStorage() public {
        vm.prank(admin);
        token.mint(alice, suint256(100e6));

        vm.prank(alice);
        assertTrue(token.transfer(bob, suint256(40e6)));

        vm.prank(alice);
        assertEq(token.balanceOf(alice), 60e6);
        vm.prank(bob);
        assertEq(token.balanceOf(bob), 40e6);
        assertEq(token.totalSupply(), 100e6);

        // The implementation's own storage stays untouched.
        vm.prank(alice);
        assertEq(implementation.balanceOf(alice), 0);
        assertEq(implementation.totalSupply(), 0);
    }

    function test_DomainSeparatorBindsToProxyAddress() public view {
        bytes32 expected = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes("Shielded USD Coin")),
                keccak256("1"),
                block.chainid,
                address(token)
            )
        );
        assertEq(token.DOMAIN_SEPARATOR(), expected);
    }

    function test_PermitThroughProxy() public {
        (address owner, uint256 ownerKey) = makeAddrAndKey("owner");

        bytes32 digest = keccak256(
            abi.encodePacked(
                "\x19\x01",
                token.DOMAIN_SEPARATOR(),
                keccak256(
                    abi.encode(
                        keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"),
                        owner,
                        bob,
                        uint256(50e6),
                        token.nonces(owner),
                        block.timestamp
                    )
                )
            )
        );
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerKey, digest);

        token.permit(owner, bob, suint256(50e6), block.timestamp, v, r, s);

        vm.prank(bob);
        assertEq(token.allowance(owner, bob), 50e6);
        assertEq(token.nonces(owner), 1);
    }

    function test_UpgradePreservesShieldedState() public {
        vm.prank(admin);
        token.mint(alice, suint256(100e6));

        SUSDC newImplementation = new SUSDC();
        vm.prank(proxyOwner);
        proxyAdmin.upgradeAndCall(ITransparentUpgradeableProxy(address(token)), address(newImplementation), "");

        vm.prank(alice);
        assertEq(token.balanceOf(alice), 100e6);
        assertEq(token.totalSupply(), 100e6);
        assertEq(token.admin(), admin);
    }

    function test_RevertWhen_NonOwnerUpgrades() public {
        SUSDC newImplementation = new SUSDC();
        vm.prank(alice);
        vm.expectRevert();
        proxyAdmin.upgradeAndCall(ITransparentUpgradeableProxy(address(token)), address(newImplementation), "");
    }
}
