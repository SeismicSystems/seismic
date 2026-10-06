// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {GasTokenRegistry} from "../src/predeploys/GasTokenRegistry.sol";

contract GasTokenRegistryTest is Test {
    GasTokenRegistry public registry;
    address public alice;
    address public tokenA;
    address public tokenB;

    GasTokenRegistry.BalanceStorageMode internal constant SHIELDED = GasTokenRegistry.BalanceStorageMode.Shielded;
    GasTokenRegistry.BalanceStorageMode internal constant PUBLIC = GasTokenRegistry.BalanceStorageMode.Public;
    address internal constant PREDEPLOY = 0x0000000000000000000000476173546f6b656e73;
    address internal constant PROTOCOL_PARAMS_OWNER = 0xd412c5Ecd343e264381fF15aFC0aD78a67B79F35;

    event TokenAdded(
        uint256 indexed index,
        address indexed token,
        uint256 balanceSlot,
        GasTokenRegistry.BalanceStorageMode balanceStorageMode,
        uint8 decimals
    );
    event TokenActivationChanged(uint256 indexed index, address indexed token, bool active);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    function setUp() public {
        registry = new GasTokenRegistry();
        alice = makeAddr("alice");
        tokenA = makeAddr("tokenA");
        tokenB = makeAddr("tokenB");
        vm.etch(tokenA, hex"00");
        vm.etch(tokenB, hex"00");
    }

    function test_ConstructorSetsOwnerAndEmptyRegistry() public view {
        assertEq(registry.owner(), address(this));
        assertEq(registry.tokenCount(), 0);
        assertEq(registry.MAX_TOKENS(), 32);
        assertEq(registry.MAX_DECIMALS(), 18);
        assertEq(uint8(SHIELDED), 0);
        assertEq(uint8(PUBLIC), 1);
    }

    function test_ConstructorEmitsOwnershipTransferred() public {
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));
        new GasTokenRegistry();
    }

    function test_AddTokenReturnsIndexAndStartsActive() public {
        assertEq(registry.addToken(tokenA, 3, SHIELDED, 6), 0);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
        assertEq(registry.tokenCount(), 1);
    }

    function test_AddTokenEmitsEventForBothModesAndEveryPrecision() public {
        for (uint8 mode = 0; mode < 2; mode++) {
            for (uint8 decimals = 0; decimals <= 18; decimals++) {
                GasTokenRegistry fresh = new GasTokenRegistry();
                GasTokenRegistry.BalanceStorageMode storageMode = GasTokenRegistry.BalanceStorageMode(mode);
                vm.expectEmit(true, true, false, true, address(fresh));
                emit TokenAdded(0, tokenA, 3, storageMode, decimals);
                assertEq(fresh.addToken(tokenA, 3, storageMode, decimals), 0);
                _assertToken(fresh, 0, tokenA, true, storageMode, decimals, 3);
            }
        }
    }

    function test_RevertWhen_DecimalsUnsupportedForBothModes() public {
        for (uint8 mode = 0; mode < 2; mode++) {
            for (uint256 decimals = 19; decimals <= 255; decimals++) {
                vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.UnsupportedDecimals.selector, uint8(decimals)));
                registry.addToken(tokenA, 3, GasTokenRegistry.BalanceStorageMode(mode), uint8(decimals));
                assertEq(registry.tokenCount(), 0);
            }
        }
    }

    function test_AddTokensPreservesInsertionOrderAndZeroSlot() public {
        assertEq(registry.addToken(tokenA, 0, SHIELDED, 0), 0);
        assertEq(registry.addToken(tokenB, type(uint256).max, PUBLIC, 18), 1);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 0, 0);
        _assertToken(registry, 1, tokenB, true, PUBLIC, 18, type(uint256).max);
        assertEq(registry.tokenCount(), 2);
    }

    function test_RevertWhen_AddTokenIsZeroAddress() public {
        vm.expectRevert(GasTokenRegistry.ZeroAddress.selector);
        registry.addToken(address(0), 3, SHIELDED, 6);
        assertEq(registry.tokenCount(), 0);
    }

    function test_RevertWhen_AddTokenHasNoCode() public {
        assertEq(alice.code.length, 0);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenHasNoCode.selector, alice));
        registry.addToken(alice, 3, SHIELDED, 6);
        assertEq(registry.tokenCount(), 0);
    }

    function test_AddTokenAcceptsDeployedProxy() public {
        ERC1967Proxy proxy = new ERC1967Proxy(tokenA, "");
        registry.addToken(address(proxy), 3, SHIELDED, 8);
        registry.deactivateToken(address(proxy));
        _assertToken(registry, 0, address(proxy), false, SHIELDED, 8, 3);
        registry.activateToken(address(proxy));
        _assertToken(registry, 0, address(proxy), true, SHIELDED, 8, 3);
    }

    function test_RevertWhen_AddTokenIsDuplicateIncludingInactive() public {
        registry.addToken(tokenA, 3, SHIELDED, 6);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenAlreadyRegistered.selector, tokenA));
        registry.addToken(tokenA, 7, PUBLIC, 18);
        registry.deactivateToken(tokenA);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenAlreadyRegistered.selector, tokenA));
        registry.addToken(tokenA, 7, PUBLIC, 0);
        _assertToken(registry, 0, tokenA, false, SHIELDED, 6, 3);
        assertEq(registry.tokenCount(), 1);
    }

    function test_RevertWhen_RegistryIsFull() public {
        uint256 limit = registry.MAX_TOKENS();
        for (uint256 i = 0; i < limit; i++) {
            address token = address(uint160(0x1000 + i));
            vm.etch(token, hex"00");
            registry.addToken(token, i, i % 2 == 0 ? SHIELDED : PUBLIC, uint8(i % 19));
        }
        assertEq(registry.tokenCount(), limit);
        registry.deactivateToken(address(0x1000));
        address extra = address(uint160(0x1000 + limit));
        vm.etch(extra, hex"00");
        vm.expectRevert(GasTokenRegistry.RegistryFull.selector);
        registry.addToken(extra, 3, SHIELDED, 6);
        assertEq(registry.tokenCount(), limit);
        registry.deactivateToken(address(uint160(0x1000 + limit - 1)));
        registry.activateToken(address(uint160(0x1000 + limit - 1)));
        _assertToken(registry, limit - 1, address(uint160(0x1000 + limit - 1)), true, PUBLIC, 12, limit - 1);
        _assertToken(registry, 0, address(0x1000), false, SHIELDED, 0, 0);
        registry.activateToken(address(0x1000));
        _assertToken(registry, 0, address(0x1000), true, SHIELDED, 0, 0);
    }

    function test_ActivationEventsAndIdempotence() public {
        registry.addToken(tokenA, 3, SHIELDED, 6);
        registry.addToken(tokenB, 7, PUBLIC, 18);
        vm.expectEmit(true, true, false, true);
        emit TokenActivationChanged(1, tokenB, false);
        registry.deactivateToken(tokenB);
        registry.deactivateToken(tokenB);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
        _assertToken(registry, 1, tokenB, false, PUBLIC, 18, 7);
        vm.expectEmit(true, true, false, true);
        emit TokenActivationChanged(1, tokenB, true);
        registry.activateToken(tokenB);
        registry.activateToken(tokenB);
        _assertToken(registry, 1, tokenB, true, PUBLIC, 18, 7);
        assertEq(registry.tokenCount(), 2);
    }

    function test_RevertWhen_TogglingUnregisteredOrZeroAddress() public {
        registry.addToken(tokenA, 3, SHIELDED, 6);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenNotRegistered.selector, tokenB));
        registry.activateToken(tokenB);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenNotRegistered.selector, tokenB));
        registry.deactivateToken(tokenB);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenNotRegistered.selector, address(0)));
        registry.activateToken(address(0));
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenNotRegistered.selector, address(0)));
        registry.deactivateToken(address(0));
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
    }

    function test_TogglingRegisteredTokenDoesNotRequireCode() public {
        registry.addToken(tokenA, 3, SHIELDED, 6);
        vm.etch(tokenA, "");
        registry.deactivateToken(tokenA);
        _assertToken(registry, 0, tokenA, false, SHIELDED, 6, 3);
        registry.activateToken(tokenA);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
    }

    function test_TransferOwnershipEmitsEventAndAllowsNewOwnerToAdminister() public {
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(this), alice);
        registry.transferOwnership(alice);
        assertEq(registry.owner(), alice);
        vm.startPrank(alice);
        registry.addToken(tokenA, 3, PUBLIC, 8);
        registry.deactivateToken(tokenA);
        registry.activateToken(tokenA);
        vm.stopPrank();
        _assertToken(registry, 0, tokenA, true, PUBLIC, 8, 3);
        _assertOwnerOnly(address(this));
    }

    function test_RevertWhen_NonOwnerAdministersEmptyRegistry() public {
        _assertOwnerOnly(alice);
        assertEq(registry.tokenCount(), 0);
        assertEq(registry.owner(), address(this));
    }

    function test_RevertWhen_TransferOwnershipToZeroAddress() public {
        vm.expectRevert(GasTokenRegistry.ZeroAddress.selector);
        registry.transferOwnership(address(0));
    }

    function test_RenounceOwnershipEmitsEventAndPreservesEntries() public {
        registry.addToken(tokenA, 3, SHIELDED, 6);
        registry.addToken(tokenB, 7, PUBLIC, 18);
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(this), address(0));
        registry.renounceOwnership();
        assertEq(registry.owner(), address(0));
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
        _assertToken(registry, 1, tokenB, true, PUBLIC, 18, 7);
        assertEq(registry.tokenCount(), 2);
        _assertOwnerOnly(address(this));
    }

    function test_ReadsArePublic() public {
        registry.addToken(tokenA, 3, SHIELDED, 6);
        registry.addToken(tokenB, 7, PUBLIC, 18);
        vm.startPrank(alice);
        assertEq(registry.tokenCount(), 2);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
        _assertToken(registry, 1, tokenB, true, PUBLIC, 18, 7);
        vm.stopPrank();
    }

    function test_StorageLayoutMatchesDirectReaderForEveryPrecisionAndMode() public {
        uint256 base = uint256(keccak256(abi.encode(uint256(1))));
        for (uint8 mode = 0; mode < 2; mode++) {
            for (uint8 decimals = 0; decimals <= 18; decimals++) {
                GasTokenRegistry fresh = new GasTokenRegistry();
                GasTokenRegistry.BalanceStorageMode storageMode = GasTokenRegistry.BalanceStorageMode(mode);
                fresh.addToken(tokenA, 0, storageMode, decimals);
                fresh.addToken(tokenB, type(uint256).max, storageMode, decimals);
                uint256 metadata = (uint256(mode) << 168) | (uint256(decimals) << 176);
                uint256 packed = uint256(uint160(tokenA)) | metadata;
                assertEq(vm.load(address(fresh), bytes32(uint256(0))), bytes32(uint256(uint160(address(this)))));
                assertEq(vm.load(address(fresh), bytes32(uint256(1))), bytes32(uint256(2)));
                assertEq(vm.load(address(fresh), bytes32(base)), bytes32(packed | (uint256(1) << 160)));
                assertEq(vm.load(address(fresh), bytes32(base + 1)), bytes32(uint256(0)));
                assertEq(
                    vm.load(address(fresh), bytes32(base + 2)),
                    bytes32(uint256(uint160(tokenB)) | metadata | (uint256(1) << 160))
                );
                assertEq(vm.load(address(fresh), bytes32(base + 3)), bytes32(type(uint256).max));
                fresh.deactivateToken(tokenA);
                assertEq(vm.load(address(fresh), bytes32(base)), bytes32(packed));
                fresh.activateToken(tokenA);
                assertEq(vm.load(address(fresh), bytes32(base)), bytes32(packed | (uint256(1) << 160)));
                _assertToken(fresh, 0, tokenA, true, storageMode, decimals, 0);
                _assertToken(fresh, 1, tokenB, true, storageMode, decimals, type(uint256).max);
            }
        }
    }

    function test_ActiveByteDecodingMatchesDirectReaderForAllValues() public {
        registry.addToken(tokenA, 3, SHIELDED, 8);
        bytes32 entrySlot = keccak256(abi.encode(uint256(1)));
        uint256 padding = uint256(type(uint72).max) << 184;
        for (uint256 mode = 0; mode < 2; mode++) {
            for (uint256 flag = 0; flag < 256; flag++) {
                uint256 word = uint256(uint160(tokenA)) | (flag << 160) | (mode << 168) | (uint256(8) << 176) | padding;
                vm.store(address(registry), entrySlot, bytes32(word));
                uint256 stored = uint256(vm.load(address(registry), entrySlot));
                assertEq(address(uint160(stored)), tokenA);
                assertEq(((stored >> 160) & 0xff) != 0, flag != 0);
                assertEq((stored >> 168) & 0xff, mode);
                assertEq((stored >> 176) & 0xff, 8);
                _assertToken(registry, 0, tokenA, flag != 0, GasTokenRegistry.BalanceStorageMode(mode), 8, 3);
            }
        }
    }

    function testFuzz_TogglingPreservesPackedMetadataAndPadding(
        uint8 activeByte,
        uint72 padding,
        bool publicBalances,
        uint256 balanceSlot,
        uint8 decimalsSeed
    ) public {
        uint8 decimals = decimalsSeed % 19;
        GasTokenRegistry.BalanceStorageMode mode = publicBalances ? PUBLIC : SHIELDED;
        registry.addToken(tokenA, balanceSlot, mode, decimals);
        registry.addToken(tokenB, 7, PUBLIC, 18);
        uint256 base = uint256(keccak256(abi.encode(uint256(1))));
        uint256 word = uint256(uint160(tokenA)) | (uint256(activeByte) << 160) | (uint256(uint8(mode)) << 168)
            | (uint256(decimals) << 176) | (uint256(padding) << 184);
        uint256 flagMask = uint256(0xff) << 160;
        vm.store(address(registry), bytes32(base), bytes32(word));
        _assertToken(registry, 0, tokenA, activeByte != 0, mode, decimals, balanceSlot);
        registry.deactivateToken(tokenA);
        assertEq(vm.load(address(registry), bytes32(base)), bytes32(word & ~flagMask));
        _assertToken(registry, 0, tokenA, false, mode, decimals, balanceSlot);
        registry.activateToken(tokenA);
        assertEq(vm.load(address(registry), bytes32(base)), bytes32((word & ~flagMask) | (uint256(1) << 160)));
        assertEq(vm.load(address(registry), bytes32(base + 1)), bytes32(balanceSlot));
        _assertToken(registry, 0, tokenA, true, mode, decimals, balanceSlot);
        _assertToken(registry, 1, tokenB, true, PUBLIC, 18, 7);
        assertEq(registry.tokenCount(), 2);
        assertEq(registry.owner(), address(this));
    }

    function test_GenesisPredeployStartsEmptyAndRequiresSeededOwner() public {
        // Installing runtime code does not execute the constructor. Fresh genesis
        // installs no token entries; registration happens through owner calls.
        bytes memory runtime =
            vm.parseJsonBytes(vm.readFile("artifacts/GasTokenRegistry.json"), ".deployedBytecode.object");
        assertEq(runtime, address(registry).code, "distributed runtime must match the pinned build");
        vm.etch(PREDEPLOY, runtime);
        GasTokenRegistry predeploy = GasTokenRegistry(PREDEPLOY);
        assertEq(predeploy.owner(), address(0));
        assertEq(predeploy.tokenCount(), 0);
        vm.expectRevert(GasTokenRegistry.OnlyOwner.selector);
        predeploy.addToken(tokenA, 3, SHIELDED, 6);
        vm.store(PREDEPLOY, bytes32(uint256(0)), bytes32(uint256(uint160(PROTOCOL_PARAMS_OWNER))));
        assertEq(predeploy.owner(), PROTOCOL_PARAMS_OWNER);
        assertEq(predeploy.tokenCount(), 0);
        vm.prank(PROTOCOL_PARAMS_OWNER);
        predeploy.addToken(tokenA, 3, SHIELDED, 6);
        _assertToken(predeploy, 0, tokenA, true, SHIELDED, 6, 3);
    }

    function test_ZeroDecimalsIsActualPrecisionNotSixDecimalDefault() public {
        registry.addToken(tokenA, 3, SHIELDED, 0);
        registry.deactivateToken(tokenA);
        registry.activateToken(tokenA);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 0, 3);
    }

    function test_RevertWhen_OldThreeArgumentApiIsCalled() public {
        (bool success,) = address(registry).call(
            abi.encodeWithSignature("addToken(address,uint256,uint8)", tokenA, uint256(3), uint8(SHIELDED))
        );
        assertFalse(success);
        assertEq(registry.tokenCount(), 0);
    }

    function testFuzz_AddTokenPreservesFullMetadata(
        address token,
        uint256 balanceSlot,
        bool publicBalances,
        uint8 decimalsSeed
    ) public {
        // Stay outside all low reserved/precompile addresses, including 0x100.
        vm.assume(uint160(token) >= 0x1000);
        if (token.code.length == 0) vm.etch(token, hex"00");
        uint8 decimals = decimalsSeed % 19;
        GasTokenRegistry.BalanceStorageMode mode = publicBalances ? PUBLIC : SHIELDED;
        registry.addToken(token, balanceSlot, mode, decimals);
        _assertToken(registry, 0, token, true, mode, decimals, balanceSlot);
        registry.deactivateToken(token);
        _assertToken(registry, 0, token, false, mode, decimals, balanceSlot);
        registry.activateToken(token);
        _assertToken(registry, 0, token, true, mode, decimals, balanceSlot);
    }

    function testFuzz_ToggleOnlyMatchingToken(uint8 countSeed, uint8 indexSeed, uint256 balanceSlot) public {
        uint256 count = uint256(countSeed) % registry.MAX_TOKENS() + 1;
        uint256 index = uint256(indexSeed) % count;
        for (uint256 i = 0; i < count; i++) {
            address token = address(uint160(0x1000 + i));
            vm.etch(token, hex"00");
            registry.addToken(token, balanceSlot ^ i, i % 2 == 0 ? SHIELDED : PUBLIC, uint8(i % 19));
        }
        address selectedToken = address(uint160(0x1000 + index));
        registry.deactivateToken(selectedToken);
        for (uint256 i = 0; i < count; i++) {
            _assertToken(
                registry,
                i,
                address(uint160(0x1000 + i)),
                i != index,
                i % 2 == 0 ? SHIELDED : PUBLIC,
                uint8(i % 19),
                balanceSlot ^ i
            );
        }
        registry.activateToken(selectedToken);
        for (uint256 i = 0; i < count; i++) {
            _assertToken(
                registry,
                i,
                address(uint160(0x1000 + i)),
                true,
                i % 2 == 0 ? SHIELDED : PUBLIC,
                uint8(i % 19),
                balanceSlot ^ i
            );
        }
        assertEq(registry.tokenCount(), count);
    }

    function testFuzz_RevertWhen_AddTokenModeIsInvalid(uint256 rawMode) public {
        vm.assume(rawMode > uint256(uint8(PUBLIC)));
        (bool success,) =
            address(registry).call(abi.encodeWithSelector(registry.addToken.selector, tokenA, 3, rawMode, 6));
        assertFalse(success);
        assertEq(registry.tokenCount(), 0);
        assertEq(registry.owner(), address(this));
    }

    function testFuzz_RevertWhen_NonOwnerMutatesRegistry(address caller) public {
        vm.assume(caller != address(this));
        vm.assume(caller != address(0));
        registry.addToken(tokenA, 3, SHIELDED, 6);
        _assertOwnerOnly(caller);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
    }

    function testFuzz_RevertWhen_TogglingUnregisteredToken(address token) public {
        registry.addToken(tokenA, 3, SHIELDED, 6);
        vm.assume(token != tokenA);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenNotRegistered.selector, token));
        registry.activateToken(token);
        vm.expectRevert(abi.encodeWithSelector(GasTokenRegistry.TokenNotRegistered.selector, token));
        registry.deactivateToken(token);
        _assertToken(registry, 0, tokenA, true, SHIELDED, 6, 3);
        assertEq(registry.tokenCount(), 1);
    }

    function _assertOwnerOnly(address caller) internal {
        vm.startPrank(caller);
        vm.expectRevert(GasTokenRegistry.OnlyOwner.selector);
        registry.addToken(tokenB, 7, PUBLIC, 18);
        vm.expectRevert(GasTokenRegistry.OnlyOwner.selector);
        registry.activateToken(tokenA);
        vm.expectRevert(GasTokenRegistry.OnlyOwner.selector);
        registry.deactivateToken(tokenA);
        vm.expectRevert(GasTokenRegistry.OnlyOwner.selector);
        registry.transferOwnership(alice);
        vm.expectRevert(GasTokenRegistry.OnlyOwner.selector);
        registry.renounceOwnership();
        vm.stopPrank();
    }

    function _assertToken(
        GasTokenRegistry target,
        uint256 index,
        address expectedToken,
        bool expectedActive,
        GasTokenRegistry.BalanceStorageMode expectedMode,
        uint8 expectedDecimals,
        uint256 expectedBalanceSlot
    ) internal view {
        (address token, bool active, GasTokenRegistry.BalanceStorageMode mode, uint8 decimals, uint256 balanceSlot) =
            target.tokens(index);
        assertEq(token, expectedToken);
        assertEq(active, expectedActive);
        assertEq(uint8(mode), uint8(expectedMode));
        assertEq(decimals, expectedDecimals);
        assertEq(balanceSlot, expectedBalanceSlot);
    }
}
