// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import {MockSRC20} from "./utils/MockSRC20.sol";
import {IDirectory} from "seismic-std-lib/interfaces/IDirectory.sol";
import {IIntelligence} from "seismic-std-lib/interfaces/IIntelligence.sol";

/// @notice Tests that SRC20 Transfer events are emitted symmetrically
///         to both sender and recipient, regardless of whether each party
///         has a registered AES key.
contract SRC20EventsTest is Test {
    MockSRC20 token;

    address constant INTELLIGENCE = 0x1000000000000000000000000000000000000005;
    address constant DIRECTORY = 0x1000000000000000000000000000000000000004;

    address sender = makeAddr("sender");
    address recipient = makeAddr("recipient");

    bytes32 constant SENDER_KEY_HASH = keccak256("sender_key");
    bytes32 constant RECIPIENT_KEY_HASH = keccak256("recipient_key");
    bytes constant SENDER_ENCRYPTED = hex"aabbccdd";
    bytes constant RECIPIENT_ENCRYPTED = hex"eeff0011";

    event Transfer(address indexed from, address indexed to, bytes32 indexed encryptKeyHash, bytes encryptedAmount);

    function setUp() public {
        // Place code at the precompile addresses so mockCall works
        vm.etch(INTELLIGENCE, hex"00");
        vm.etch(DIRECTORY, hex"00");

        // Mock Intelligence.encryptToProviders to return empty arrays (no providers)
        vm.mockCall(
            INTELLIGENCE,
            abi.encodeWithSelector(IIntelligence.encryptToProviders.selector),
            abi.encode(new bytes32[](0), new bytes[](0))
        );

        // Default: any checkHasKey call returns false (covers address(0), test contract, etc.)
        vm.mockCall(DIRECTORY, abi.encodeWithSelector(IDirectory.checkHasKey.selector), abi.encode(false));

        token = new MockSRC20("Token", "TKN", 18);
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: BOTH PARTIES HAVE KEYS
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsEventToBothPartiesWhenBothHaveKeys() public {
        _mockSenderHasKey(true);
        _mockRecipientHasKey(true);

        token.mint(sender, suint256(1e18));

        // Expect sender event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, SENDER_KEY_HASH, SENDER_ENCRYPTED);
        // Expect recipient event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, RECIPIENT_KEY_HASH, RECIPIENT_ENCRYPTED);

        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: ONLY SENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsEncryptedEventToSenderWhenOnlySenderHasKey() public {
        _mockSenderHasKey(true);
        _mockRecipientHasKey(false);

        token.mint(sender, suint256(1e18));

        // Expect sender encrypted event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, SENDER_KEY_HASH, SENDER_ENCRYPTED);
        // Expect recipient zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));

        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: ONLY RECIPIENT HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsEncryptedEventToRecipientWhenOnlyRecipientHasKey() public {
        _mockSenderHasKey(false);
        _mockRecipientHasKey(true);

        token.mint(sender, suint256(1e18));

        // Expect sender zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));
        // Expect recipient encrypted event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, RECIPIENT_KEY_HASH, RECIPIENT_ENCRYPTED);

        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                    TRANSFER: NEITHER PARTY HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferEmitsZeroHashEventsToBothPartiesWhenNeitherHasKey() public {
        _mockSenderHasKey(false);
        _mockRecipientHasKey(false);

        token.mint(sender, suint256(1e18));

        // Expect sender zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));
        // Expect recipient zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));

        vm.prank(sender);
        token.transfer(recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: BOTH PARTIES HAVE KEYS
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsEventToBothPartiesWhenBothHaveKeys() public {
        _mockSenderHasKey(true);
        _mockRecipientHasKey(true);

        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        // Expect sender event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, SENDER_KEY_HASH, SENDER_ENCRYPTED);
        // Expect recipient event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, RECIPIENT_KEY_HASH, RECIPIENT_ENCRYPTED);

        token.transferFrom(sender, recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: ONLY SENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsEncryptedEventToSenderWhenOnlySenderHasKey() public {
        _mockSenderHasKey(true);
        _mockRecipientHasKey(false);

        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, SENDER_KEY_HASH, SENDER_ENCRYPTED);
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));

        token.transferFrom(sender, recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: ONLY RECIPIENT HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsEncryptedEventToRecipientWhenOnlyRecipientHasKey() public {
        _mockSenderHasKey(false);
        _mockRecipientHasKey(true);

        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, RECIPIENT_KEY_HASH, RECIPIENT_ENCRYPTED);

        token.transferFrom(sender, recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                TRANSFER FROM: NEITHER PARTY HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_TransferFromEmitsZeroHashEventsToBothPartiesWhenNeitherHasKey() public {
        _mockSenderHasKey(false);
        _mockRecipientHasKey(false);

        token.mint(sender, suint256(1e18));
        vm.prank(sender);
        token.approve(address(this), suint256(1e18));

        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, recipient, bytes32(0), bytes(""));

        token.transferFrom(sender, recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                    MINT: RECIPIENT HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_MintEmitsEncryptedEventToRecipientWhenRecipientHasKey() public {
        _mockHasKey(recipient, true, RECIPIENT_KEY_HASH, RECIPIENT_ENCRYPTED);
        _mockHasKey(address(0), false, bytes32(0), bytes(""));

        // Expect sender (address(0)) zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(address(0), recipient, bytes32(0), bytes(""));
        // Expect recipient encrypted event
        vm.expectEmit(true, true, true, true);
        emit Transfer(address(0), recipient, RECIPIENT_KEY_HASH, RECIPIENT_ENCRYPTED);

        token.mint(recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                    MINT: RECIPIENT HAS NO KEY
    //////////////////////////////////////////////////////////////*/

    function test_MintEmitsZeroHashEventsWhenRecipientHasNoKey() public {
        _mockHasKey(recipient, false, bytes32(0), bytes(""));
        _mockHasKey(address(0), false, bytes32(0), bytes(""));

        // Expect sender (address(0)) zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(address(0), recipient, bytes32(0), bytes(""));
        // Expect recipient zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(address(0), recipient, bytes32(0), bytes(""));

        token.mint(recipient, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                    BURN: SENDER HAS KEY
    //////////////////////////////////////////////////////////////*/

    function test_BurnEmitsEncryptedEventToSenderWhenSenderHasKey() public {
        _mockHasKey(sender, true, SENDER_KEY_HASH, SENDER_ENCRYPTED);
        _mockHasKey(address(0), false, bytes32(0), bytes(""));

        token.mint(sender, suint256(1e18));

        // Expect sender encrypted event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, address(0), SENDER_KEY_HASH, SENDER_ENCRYPTED);
        // Expect recipient (address(0)) zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, address(0), bytes32(0), bytes(""));

        token.burn(sender, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                    BURN: SENDER HAS NO KEY
    //////////////////////////////////////////////////////////////*/

    function test_BurnEmitsZeroHashEventsWhenSenderHasNoKey() public {
        _mockHasKey(sender, false, bytes32(0), bytes(""));
        _mockHasKey(address(0), false, bytes32(0), bytes(""));

        token.mint(sender, suint256(1e18));

        // Expect sender zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, address(0), bytes32(0), bytes(""));
        // Expect recipient (address(0)) zero-hash event
        vm.expectEmit(true, true, true, true);
        emit Transfer(sender, address(0), bytes32(0), bytes(""));

        token.burn(sender, suint256(1e18));
    }

    /*//////////////////////////////////////////////////////////////
                            HELPERS
    //////////////////////////////////////////////////////////////*/

    function _mockSenderHasKey(bool hasKey) internal {
        if (hasKey) {
            _mockHasKey(sender, true, SENDER_KEY_HASH, SENDER_ENCRYPTED);
        } else {
            _mockHasKey(sender, false, bytes32(0), bytes(""));
        }
    }

    function _mockRecipientHasKey(bool hasKey) internal {
        if (hasKey) {
            _mockHasKey(recipient, true, RECIPIENT_KEY_HASH, RECIPIENT_ENCRYPTED);
        } else {
            _mockHasKey(recipient, false, bytes32(0), bytes(""));
        }
    }

    function _mockHasKey(address addr, bool hasKey, bytes32 keyHash, bytes memory encrypted) internal {
        vm.mockCall(DIRECTORY, abi.encodeWithSelector(IDirectory.checkHasKey.selector, addr), abi.encode(hasKey));
        if (hasKey) {
            vm.mockCall(DIRECTORY, abi.encodeWithSelector(IDirectory.keyHash.selector, addr), abi.encode(keyHash));
            vm.mockCall(DIRECTORY, abi.encodeWithSelector(IDirectory.encrypt.selector, addr), abi.encode(encrypted));
        }
    }
}
