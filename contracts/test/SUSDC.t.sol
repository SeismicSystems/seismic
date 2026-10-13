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

/// @dev xERC20 (draft ERC-7281, defi-wonderland reference) as seen by a bridge router (e.g. Hyperlane HypXERC20) and
///      by the Hyperlane SDK, which reads the limit getters before every transfer.
///      Exercises selectors mint(address,uint256) 0x40c10f19, burn(address,uint256) 0x9dc29fac,
///      mintingCurrentLimitOf(address) 0x651fd268 and friends.
interface IXERC20MintBurn {
    function mint(address to, uint256 amount) external;
    function burn(address from, uint256 amount) external;
    function setLimits(address bridge, uint256 mintingLimit, uint256 burningLimit) external;
    function owner() external returns (address);
    function mintingCurrentLimitOf(address bridge) external view returns (uint256);
    function burningCurrentLimitOf(address bridge) external view returns (uint256);
    function mintingMaxLimitOf(address bridge) external view returns (uint256);
    function burningMaxLimitOf(address bridge) external view returns (uint256);
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

    /*//////////////////////////////////////////////////////////////
                              BRIDGE ROLE
    //////////////////////////////////////////////////////////////*/

    address bridge = makeAddr("bridge");

    function test_SetBridgeEmitsAndStores() public {
        vm.expectEmit(true, true, false, false);
        emit SUSDC.BridgeUpdated(address(0), bridge);
        token.setBridge(bridge);
        assertEq(token.bridge(), bridge);
    }

    function test_RevertWhen_NonAdminSetsBridge() public {
        vm.prank(alice);
        vm.expectRevert("SUSDC: caller is not admin");
        token.setBridge(bridge);
    }

    function test_BridgeMintsAndBurnsViaXERC20Selectors() public {
        token.setBridge(bridge);
        token.setLimits(bridge, 1_000e6, 1_000e6);
        IXERC20MintBurn xerc20 = IXERC20MintBurn(address(token));

        vm.prank(bridge);
        xerc20.mint(alice, 100e6);
        assertEq(token.totalSupply(), 100e6);
        vm.prank(alice);
        assertEq(erc20.balanceOf(alice), 100e6);

        // Router burns from the transfer initiator without an allowance, like admin burn.
        vm.prank(bridge);
        xerc20.burn(alice, 30e6);
        assertEq(token.totalSupply(), 70e6);
        vm.prank(alice);
        assertEq(erc20.balanceOf(alice), 70e6);
    }

    function test_BridgeCanUseShieldedMintBurn() public {
        token.setBridge(bridge);
        token.setLimits(bridge, 1_000e6, 1_000e6);
        vm.prank(bridge);
        token.mint(alice, suint256(5e6));
        vm.prank(bridge);
        token.burn(alice, suint256(2e6));
        assertEq(token.totalSupply(), 3e6);
    }

    function test_AdminCanUseXERC20Selectors() public {
        IXERC20MintBurn(address(token)).mint(alice, 1e6);
        IXERC20MintBurn(address(token)).burn(alice, 1e6);
        assertEq(token.totalSupply(), 0);
    }

    function test_RevertWhen_NonBridgeMintsOrBurns() public {
        token.setBridge(bridge);
        IXERC20MintBurn xerc20 = IXERC20MintBurn(address(token));

        vm.prank(alice);
        vm.expectRevert("SUSDC: caller is not admin or bridge");
        xerc20.mint(alice, 1e6);

        vm.prank(alice);
        vm.expectRevert("SUSDC: caller is not admin or bridge");
        xerc20.burn(alice, 1e6);

        vm.prank(alice);
        vm.expectRevert("SUSDC: caller is not admin or bridge");
        token.mint(alice, suint256(1e6));
    }

    function test_ClearedBridgeLosesMintRights() public {
        token.setBridge(bridge);
        token.setLimits(bridge, 1_000e6, 1_000e6);
        token.setBridge(address(0));
        vm.prank(bridge);
        vm.expectRevert("SUSDC: caller is not admin or bridge");
        IXERC20MintBurn(address(token)).mint(alice, 1e6);
    }

    /*//////////////////////////////////////////////////////////////
                      xERC20 BRIDGE LIMITS
    //////////////////////////////////////////////////////////////*/

    IXERC20MintBurn xerc20;

    function _setupLimitedBridge(uint256 mintLimit, uint256 burnLimit) internal {
        token.setBridge(bridge);
        token.setLimits(bridge, mintLimit, burnLimit);
        xerc20 = IXERC20MintBurn(address(token));
    }

    function test_OwnerIsAdmin() public {
        assertEq(IXERC20MintBurn(address(token)).owner(), token.admin());
    }

    function test_RevertWhen_BridgeMintsWithoutLimits() public {
        // Fail-closed: setBridge alone grants no allowance.
        token.setBridge(bridge);
        assertEq(token.mintingCurrentLimitOf(bridge), 0);
        vm.prank(bridge);
        vm.expectRevert(SUSDC.IXERC20_NotHighEnoughLimits.selector);
        IXERC20MintBurn(address(token)).mint(alice, 1);
        vm.prank(bridge);
        vm.expectRevert(SUSDC.IXERC20_NotHighEnoughLimits.selector);
        token.mint(alice, suint256(1));
    }

    function test_SetLimitsEmitsAndExposesGetters() public {
        vm.expectEmit(true, false, false, true);
        emit SUSDC.BridgeLimitsSet(100e6, 50e6, bridge);
        token.setLimits(bridge, 100e6, 50e6);

        assertEq(token.mintingMaxLimitOf(bridge), 100e6);
        assertEq(token.burningMaxLimitOf(bridge), 50e6);
        assertEq(token.mintingCurrentLimitOf(bridge), 100e6);
        assertEq(token.burningCurrentLimitOf(bridge), 50e6);
        // Unrelated addresses have no allowance.
        assertEq(token.mintingCurrentLimitOf(alice), 0);
    }

    function test_RevertWhen_NonAdminSetsLimits() public {
        vm.prank(alice);
        vm.expectRevert("SUSDC: caller is not admin");
        token.setLimits(bridge, 1, 1);
    }

    function test_RevertWhen_LimitsTooHigh() public {
        vm.expectRevert(SUSDC.IXERC20_LimitsTooHigh.selector);
        token.setLimits(bridge, type(uint256).max / 2 + 1, 0);
        vm.expectRevert(SUSDC.IXERC20_LimitsTooHigh.selector);
        token.setLimits(bridge, 0, type(uint256).max / 2 + 1);
    }

    function test_BridgeMintConsumesMintingLimit() public {
        _setupLimitedBridge(100e6, 100e6);

        vm.prank(bridge);
        xerc20.mint(alice, 60e6);
        assertEq(token.mintingCurrentLimitOf(bridge), 40e6);
        // Burning allowance is independent.
        assertEq(token.burningCurrentLimitOf(bridge), 100e6);

        vm.prank(bridge);
        vm.expectRevert(SUSDC.IXERC20_NotHighEnoughLimits.selector);
        xerc20.mint(alice, 40e6 + 1);

        vm.prank(bridge);
        xerc20.mint(alice, 40e6);
        assertEq(token.mintingCurrentLimitOf(bridge), 0);
    }

    function test_BridgeBurnConsumesBurningLimit() public {
        _setupLimitedBridge(100e6, 30e6);
        token.mint(alice, suint256(100e6)); // admin mint, unlimited

        vm.prank(bridge);
        xerc20.burn(alice, 30e6);
        assertEq(token.burningCurrentLimitOf(bridge), 0);
        assertEq(token.mintingCurrentLimitOf(bridge), 100e6);

        vm.prank(bridge);
        vm.expectRevert(SUSDC.IXERC20_NotHighEnoughLimits.selector);
        xerc20.burn(alice, 1);
    }

    function test_ShieldedOverloadsShareTheLimit() public {
        _setupLimitedBridge(100e6, 100e6);
        vm.prank(bridge);
        token.mint(alice, suint256(70e6));
        assertEq(token.mintingCurrentLimitOf(bridge), 30e6);
        vm.prank(bridge);
        vm.expectRevert(SUSDC.IXERC20_NotHighEnoughLimits.selector);
        xerc20.mint(alice, 30e6 + 1);
    }

    function test_LimitReplenishesLinearlyOverOneDay() public {
        _setupLimitedBridge(100e6, 100e6);
        vm.prank(bridge);
        xerc20.mint(alice, 100e6);
        assertEq(token.mintingCurrentLimitOf(bridge), 0);

        vm.warp(block.timestamp + 12 hours);
        // rate = max / 1 day, so half a day replenishes half (rounding from integer rate).
        uint256 half = token.mintingCurrentLimitOf(bridge);
        assertApproxEqAbs(half, 50e6, 1 days);

        vm.warp(block.timestamp + 12 hours);
        assertEq(token.mintingCurrentLimitOf(bridge), 100e6);

        // Never exceeds max.
        vm.warp(block.timestamp + 30 days);
        assertEq(token.mintingCurrentLimitOf(bridge), 100e6);
    }

    function test_RaisingLimitRaisesCurrentByTheDifference() public {
        _setupLimitedBridge(100e6, 100e6);
        vm.prank(bridge);
        xerc20.mint(alice, 80e6); // current 20
        token.setLimits(bridge, 150e6, 100e6);
        assertEq(token.mintingCurrentLimitOf(bridge), 70e6);
        assertEq(token.mintingMaxLimitOf(bridge), 150e6);
    }

    function test_LoweringLimitLowersCurrentFlooredAtZero() public {
        _setupLimitedBridge(100e6, 100e6);
        vm.prank(bridge);
        xerc20.mint(alice, 80e6); // current 20
        token.setLimits(bridge, 50e6, 100e6); // diff 50 > current 20
        assertEq(token.mintingCurrentLimitOf(bridge), 0);
        token.setLimits(bridge, 60e6, 100e6); // +10
        assertEq(token.mintingCurrentLimitOf(bridge), 10e6);
    }

    function test_ZeroLimitDisablesBridgeMinting() public {
        _setupLimitedBridge(100e6, 100e6);
        token.setLimits(bridge, 0, 0);
        vm.prank(bridge);
        vm.expectRevert(SUSDC.IXERC20_NotHighEnoughLimits.selector);
        xerc20.mint(alice, 1);
    }

    function test_AdminIsNeverLimited() public {
        _setupLimitedBridge(1, 1);
        IXERC20MintBurn(address(token)).mint(alice, 1_000_000e6);
        token.mint(alice, suint256(1_000_000e6));
        IXERC20MintBurn(address(token)).burn(alice, 2_000_000e6);
        assertEq(token.totalSupply(), 0);
        // Admin usage does not touch the bridge's allowance.
        assertEq(token.mintingCurrentLimitOf(bridge), 1);
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

    function test_BridgeRoleLivesInProxyStorage() public {
        address bridge = makeAddr("bridge");
        vm.prank(admin);
        token.setBridge(bridge);
        vm.prank(admin);
        token.setLimits(bridge, 1_000e6, 1_000e6);
        assertEq(token.bridge(), bridge);
        assertEq(implementation.bridge(), address(0));
        assertEq(implementation.mintingCurrentLimitOf(bridge), 0);

        vm.prank(bridge);
        IXERC20MintBurn(address(token)).mint(alice, 10e6);
        assertEq(token.totalSupply(), 10e6);

        // Survives upgrade: `bridge` and `bridgeLimits` are appended after `paused`.
        SUSDC newImplementation = new SUSDC();
        vm.prank(proxyOwner);
        proxyAdmin.upgradeAndCall(ITransparentUpgradeableProxy(address(token)), address(newImplementation), "");
        assertEq(token.bridge(), bridge);
        assertEq(token.mintingMaxLimitOf(bridge), 1_000e6);
        assertEq(token.mintingCurrentLimitOf(bridge), 990e6);
    }

    function test_RevertWhen_NonOwnerUpgrades() public {
        SUSDC newImplementation = new SUSDC();
        vm.prank(alice);
        vm.expectRevert();
        proxyAdmin.upgradeAndCall(ITransparentUpgradeableProxy(address(token)), address(newImplementation), "");
    }
}
