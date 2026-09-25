// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title KeyRotationRegistry
/// @notice The on-chain schedule of purpose-key rotations: an authorized admin
///         announces that the network's key epoch advances at a future activation
///         block, and every node fetches the new epoch's keys from its local key
///         custodian during the announcement window (announce-then-activate).
/// @dev Nodes do not call this contract: seismic-reth reads its **raw storage
///      slots** and filters its event by the raw topic (see `reth-seismic-keys`'s
///      `registry` module in the seismic-reth repo, and
///      `docs/design/purpose-key-rotation.md` there). The storage layout, entry
///      packing, and event signature are therefore consensus-frozen against that
///      mirror — changing any of them is a coordinated cross-repo change, never a
///      refactor. This is also why the contract uses plain declaration-order
///      storage (like ProtocolParams, whose slot 0 the ops-auth layer reads raw)
///      rather than ERC-7201 namespaced storage:
///
///        slot 0             : admin (address)
///        slot 1             : rotations.length
///        keccak256(1) + i   : rotations[i], one word packing
///                             (epoch | activationBlock << 64 | announcedAtBlock << 128)
///        slot 2             : minActivationDelay (uint64)
///
///      Scheduling invariants enforced here keep the node-side logic trivial:
///      epochs are dense (entry `i` is epoch `i + 1`; epoch 0 is the genesis epoch
///      and never appears in the array), activations strictly increase, at most one
///      rotation is pending at a time, and every activation is at least
///      `minActivationDelay` blocks past its announcement — the window in which
///      nodes fetch keys, wallets observe the pending rotation (via
///      `seismic_getKeyEpochInfo`) and switch at the boundary, and the announcement
///      becomes final. Activation is by block number, not timestamp: the host wall
///      clock is untrusted in TEE deployments, and block-number activation makes
///      the epoch of any block reorg-invariant by construction.
///
///      This contract is installed as a genesis predeploy at
///      0x1000000000000000000000000000000000000007; on genesis networks the
///      constructor does not run and `admin` (slot 0) plus `minActivationDelay`
///      (slot 2) are written directly into genesis storage by deploy tooling. The
///      constructor exists for dev/test deployments only. With
///      `minActivationDelay` unset the registry fails closed: no rotation can be
///      announced.
contract KeyRotationRegistry {
    /// @notice One announced rotation. Field order is consensus-frozen: Solidity
    ///         packs the first declared field into the low-order bits of the slot
    ///         word, which is exactly how nodes decode it.
    struct Rotation {
        /// The epoch this rotation activates; always its array index + 1.
        uint64 epoch;
        /// The first block executed with this epoch's keys.
        uint64 activationBlock;
        /// The block at which this rotation was announced.
        uint64 announcedAtBlock;
    }

    /// @notice The announce authority. Slot 0; genesis-initialized.
    address public admin;

    /// @notice The append-only rotation history. Slot 1; empty until the first
    ///         announcement (every block is epoch 0 until then).
    Rotation[] public rotations;

    /// @notice Minimum number of blocks between an announcement and its activation.
    ///         Slot 2; genesis-initialized per network. Zero means unconfigured and
    ///         disables announcements entirely (fail closed).
    uint64 public minActivationDelay;

    /// @notice Emitted for every announced rotation. Nodes use this as a
    ///         low-latency hint only (storage is their source of truth), matching
    ///         on the raw topic `keccak256("RotationAnnounced(uint64,uint64)")` —
    ///         the signature is consensus-frozen.
    /// @param epoch The epoch that will activate.
    /// @param activationBlock The first block executed with the new epoch's keys.
    event RotationAnnounced(uint64 indexed epoch, uint64 activationBlock);

    /// @notice Emitted when the announce authority changes.
    /// @param previousAdmin The address of the previous admin.
    /// @param newAdmin The address of the new admin.
    event AdminTransferred(address indexed previousAdmin, address indexed newAdmin);

    error OnlyAdmin();
    error ZeroAddress();
    /// @notice `minActivationDelay` is unset; the registry refuses announcements.
    error DelayNotConfigured();
    /// @notice A rotation is already pending; at most one may be in flight.
    error RotationPending(uint64 pendingEpoch, uint64 pendingActivationBlock);
    /// @notice The requested activation is closer than `minActivationDelay` blocks.
    error ActivationTooSoon(uint64 activationBlock, uint256 earliestAllowed);

    modifier onlyAdmin() {
        if (msg.sender != admin) revert OnlyAdmin();
        _;
    }

    /// @notice Constructs the registry for dev/test deployments. Genesis predeploy
    ///         installation bypasses this entirely and writes slots 0 and 2
    ///         directly.
    /// @param initialAdmin The announce authority.
    /// @param initialMinActivationDelay The per-network announcement delay floor.
    constructor(address initialAdmin, uint64 initialMinActivationDelay) {
        if (initialAdmin == address(0)) revert ZeroAddress();
        admin = initialAdmin;
        minActivationDelay = initialMinActivationDelay;
        emit AdminTransferred(address(0), initialAdmin);
    }

    /// @notice Announces the next key rotation: epoch `rotations.length + 1`
    ///         activates at `activationBlock`.
    /// @dev Enforces the scheduling invariants nodes rely on: configured delay
    ///      (fail closed), no rotation already pending, and activation at least
    ///      `minActivationDelay` blocks in the future — which also makes
    ///      activations strictly increasing, since the previous rotation must have
    ///      activated (its activation is <= block.number) before a new one is
    ///      accepted.
    /// @param activationBlock The first block to execute with the new epoch's keys.
    function announceRotation(uint64 activationBlock) external onlyAdmin {
        if (minActivationDelay == 0) revert DelayNotConfigured();

        uint256 count = rotations.length;
        if (count > 0) {
            Rotation storage last = rotations[count - 1];
            if (last.activationBlock > block.number) {
                revert RotationPending(last.epoch, last.activationBlock);
            }
        }

        uint256 earliestAllowed = block.number + minActivationDelay;
        if (activationBlock < earliestAllowed) {
            revert ActivationTooSoon(activationBlock, earliestAllowed);
        }

        uint64 epoch = uint64(count) + 1;
        rotations.push(
            Rotation({epoch: epoch, activationBlock: activationBlock, announcedAtBlock: uint64(block.number)})
        );
        emit RotationAnnounced(epoch, activationBlock);
    }

    /// @notice Transfers the announce authority.
    /// @param newAdmin The new admin; must not be the zero address (there is no
    ///        renounce — an admin-less registry could never rotate again).
    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ZeroAddress();
        address previousAdmin = admin;
        admin = newAdmin;
        emit AdminTransferred(previousAdmin, newAdmin);
    }

    /// @notice Number of announced rotations (the current maximum epoch).
    function rotationCount() external view returns (uint256) {
        return rotations.length;
    }

    /// @notice The key epoch active at the current block: the greatest announced
    ///         epoch whose activation is at or before `block.number`, or 0 if none.
    function currentEpoch() external view returns (uint64) {
        return epochAtBlock(uint64(block.number));
    }

    /// @notice The announced-but-not-yet-activated rotation, if any.
    /// @return exists Whether a rotation is pending.
    /// @return epoch The pending epoch (0 when none).
    /// @return activationBlock Its activation block (0 when none).
    function pendingRotation() external view returns (bool exists, uint64 epoch, uint64 activationBlock) {
        uint256 count = rotations.length;
        if (count > 0) {
            Rotation storage last = rotations[count - 1];
            if (last.activationBlock > block.number) {
                return (true, last.epoch, last.activationBlock);
            }
        }
        return (false, 0, 0);
    }

    /// @notice The key epoch active for `blockNumber` — the normative
    ///         epoch-for-block function nodes implement over raw storage, exposed
    ///         here for tooling. O(rotations) walking backward; rotations are rare
    ///         operator events, and consensus code never calls this.
    /// @param blockNumber The block to evaluate.
    function epochAtBlock(uint64 blockNumber) public view returns (uint64) {
        for (uint256 i = rotations.length; i > 0; i--) {
            Rotation storage rotation = rotations[i - 1];
            if (rotation.activationBlock <= blockNumber) {
                return rotation.epoch;
            }
        }
        return 0;
    }
}
