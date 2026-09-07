// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {ShieldedDelegationAccount} from "seismic-std-lib/ShieldedDelegationAccount.sol";
import {IShieldedDelegationAccount} from "seismic-std-lib/interfaces/IShieldedDelegationAccount.sol";
import {SDAWithERC1271} from "../../src/experiments/SDAWithERC1271.sol";

/// @notice EXPERIMENT — measures what adding ERC-1271 to ShieldedDelegationAccount would cost.
///
/// Cast:
///   ALICE — account owner. Registers BOB's EOA as a Secp256k1 session key (the
///           "add my colleague / my other device" flow), scoped to a spend limit.
///   BOB   — a human wallet. Later 7702-delegates his own EOA to an SDA, entirely
///           independently of Alice.
///   K     — an ephemeral session key BOB grants to some dapp on HIS OWN account,
///           tightly scoped there (small limit, short expiry).
///   EVE   — steals K out of the dapp's browser storage.
///
/// The three phases below correspond to the three states of the codebase.
contract ERC1271TransitiveTrustTest is Test {
    ShieldedDelegationAccount aliceImpl;
    ShieldedDelegationAccount bobPlainImpl;
    SDAWithERC1271 bobUpgradedImpl;

    uint256 aliceRootPk = 0xA11CE;
    uint256 bobPk = 0xB0B;
    uint256 kPk = 0xC0FFEE;

    address alice;
    address bob;
    address kAddr;

    address victimSink = address(0xDEAD);

    function setUp() public {
        alice = vm.addr(aliceRootPk);
        bob = vm.addr(bobPk);
        kAddr = vm.addr(kPk);

        aliceImpl = new ShieldedDelegationAccount();
        bobPlainImpl = new ShieldedDelegationAccount();
        bobUpgradedImpl = new SDAWithERC1271();

        // Alice's EOA is 7702-delegated to a stock SDA.
        vm.etch(alice, address(aliceImpl).code);
        vm.deal(alice, 100 ether);

        // Alice authorizes BOB'S ADDRESS as a Secp256k1 session key, capped at 10 ether.
        // authorizeKey is onlySelf, so it must come from the account itself.
        vm.prank(alice);
        ShieldedDelegationAccount(payable(alice)).authorizeKey(
            IShieldedDelegationAccount.KeyType.Secp256k1, abi.encode(bob), 0, 10 ether
        );
    }

    /// Builds the calls blob multiSend expects: operation(uint8) ‖ to(address) ‖ value(uint256)
    /// ‖ dataLength(uint256) ‖ data. One plain value transfer, no calldata.
    function _transferCall(address to, uint256 value) internal pure returns (bytes memory) {
        return abi.encodePacked(uint8(0), to, value, uint256(0));
    }

    /// Reproduces SDA's Execute digest for a given account (its own domain separator).
    function _executeDigest(address account, uint256 keyNonce, bytes memory calls) internal view returns (bytes32) {
        bytes32 domainSeparator = ShieldedDelegationAccount(payable(account)).getDomainSeparator();
        bytes32 structHash =
            keccak256(abi.encode(keccak256("Execute(uint256 nonce,bytes cipher)"), keyNonce, keccak256(calls)));
        return keccak256(abi.encodePacked("\x19\x01", domainSeparator, structHash));
    }

    function _sign(uint256 pk, bytes32 digest) internal pure returns (bytes memory) {
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, digest);
        return abi.encodePacked(r, s, v);
    }

    // ─────────────────────────────────────────────────────────────────────
    // PHASE 1 — today, Bob is a plain EOA. Everything works as designed.
    // ─────────────────────────────────────────────────────────────────────
    function test_phase1_plainEoaSessionKeyWorks() public {
        bytes memory calls = _transferCall(victimSink, 1 ether);
        bytes32 digest = _executeDigest(alice, 0, calls);
        bytes memory sig = _sign(bobPk, digest);

        uint256 before = victimSink.balance;
        ShieldedDelegationAccount(payable(alice)).execute(0, calls, sig, 1);

        // extcodesize(bob) == 0 → Solady took the ecrecover branch → key valid.
        assertEq(victimSink.balance - before, 1 ether, "bob's session key should work");
    }

    // ─────────────────────────────────────────────────────────────────────
    // PHASE 2 — Bob delegates his OWN wallet. Alice's session key silently dies.
    // This is the availability footgun, and it exists on main TODAY.
    // ─────────────────────────────────────────────────────────────────────
    function test_phase2_bobDelegating_breaksAlicesSessionKey() public {
        // Bob 7702-delegates to a stock SDA — nothing to do with Alice.
        vm.etch(bob, address(bobPlainImpl).code);

        bytes memory calls = _transferCall(victimSink, 1 ether);
        bytes32 digest = _executeDigest(alice, 0, calls);
        bytes memory sig = _sign(bobPk, digest); // same key, same math as phase 1

        // extcodesize(bob) != 0 → ecrecover is SKIPPED → staticcall to a missing
        // isValidSignature → revert swallowed → false → SDA's own require fires.
        vm.expectRevert("invalid signature");
        ShieldedDelegationAccount(payable(alice)).execute(0, calls, sig, 1);
    }

    // ─────────────────────────────────────────────────────────────────────
    // PHASE 3 — Bob's account implements ERC-1271 with the all-keys policy.
    // Phase 2 heals, and the transitive-trust flow opens.
    // ─────────────────────────────────────────────────────────────────────
    function test_phase3a_erc1271_healsTheAvailabilityBreak() public {
        vm.etch(bob, address(bobUpgradedImpl).code);
        // No key registration needed: the root-key path recovers to address(this).

        bytes memory calls = _transferCall(victimSink, 1 ether);
        bytes32 digest = _executeDigest(alice, 0, calls);
        bytes memory sig = _sign(bobPk, digest);

        uint256 before = victimSink.balance;
        ShieldedDelegationAccount(payable(alice)).execute(0, calls, sig, 1);
        assertEq(victimSink.balance - before, 1 ether, "1271 should heal the phase-2 break");
    }

    function test_phase3b_ATTACK_stolenEphemeralKeyDrainsAlice() public {
        vm.etch(bob, address(bobUpgradedImpl).code);

        // Bob grants K on HIS OWN account: 1 wei limit, expires in an hour.
        // Scoped as tightly as the session-key UX allows.
        vm.prank(bob);
        SDAWithERC1271(payable(bob)).authorizeKey(
            IShieldedDelegationAccount.KeyType.Secp256k1, abi.encode(kAddr), uint40(block.timestamp + 1 hours), 1 wei
        );

        // EVE steals K from the dapp and signs a digest for ALICE'S account —
        // draining Alice's full 10 ether allowance, not Bob's 1 wei.
        bytes memory calls = _transferCall(victimSink, 10 ether);
        bytes32 digest = _executeDigest(alice, 0, calls);
        bytes memory sigFromStolenK = _sign(kPk, digest);

        uint256 before = victimSink.balance;
        ShieldedDelegationAccount(payable(alice)).execute(0, calls, sigFromStolenK, 1);

        // K was never authorized on Alice's account. Alice authorized BOB.
        // Bob's account vouched for K against a digest it could not interpret.
        assertEq(victimSink.balance - before, 10 ether, "ATTACK: stolen ephemeral key drained Alice");
    }

    /// Separate hazard found while building this: an account that registers its OWN address
    /// as a Secp256k1 key makes isValidSignature self-referential. The key loop hands
    /// address(this) to SignatureCheckerLib, which sees code and staticcalls back into
    /// isValidSignature — recursing until the call-depth limit unwinds it. Not a theft path
    /// (it fails closed), but it burns the caller's gas and is a trivially reachable
    /// misconfiguration, so a real implementation must skip keys equal to address(this).
    function test_selfReferentialKey_recursesUntilDepthLimit() public {
        vm.etch(bob, address(bobUpgradedImpl).code);

        vm.prank(bob);
        SDAWithERC1271(payable(bob)).authorizeKey(
            IShieldedDelegationAccount.KeyType.Secp256k1, abi.encode(bob), 0, type(uint256).max
        );

        // Signed by a key that is NOT bob, so the root path misses and the loop is forced
        // to consult the self-referential entry.
        bytes32 digest = keccak256("anything");
        bytes memory sig = _sign(kPk, digest);

        assertEq(
            SDAWithERC1271(payable(bob)).isValidSignature(digest, sig),
            bytes4(0xffffffff),
            "self-referential key should fail closed, not validate"
        );
    }

    /// Control: without the 1271 upgrade the same attack is dead on arrival.
    function test_phase3b_control_attackFailsWithoutErc1271() public {
        vm.etch(bob, address(bobPlainImpl).code);

        bytes memory calls = _transferCall(victimSink, 10 ether);
        bytes32 digest = _executeDigest(alice, 0, calls);
        bytes memory sigFromStolenK = _sign(kPk, digest);

        vm.expectRevert("invalid signature");
        ShieldedDelegationAccount(payable(alice)).execute(0, calls, sigFromStolenK, 1);
    }
}
