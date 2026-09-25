// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title MeasurementRegistry
/// @notice Stores the admission status of opaque, schema-separated measurement IDs.
/// @dev An off-chain policy compiler derives two outputs from each complete policy
///      document: the set of admitted IDs and the SHA-256 hash of the exact document
///      bytes. For a tuple-based schema, each ID represents a conjunction of exact
///      observed values (for example, PCR4 == a AND PCR9 == b AND PCR11 == c), and
///      the accepted-ID set is the disjunction (OR) of those concrete clauses. Any
///      policy alternatives are expanded into concrete IDs before reaching this
///      contract.
///
///      At genesis, deploy tooling writes the compiled IDs and document hash directly
///      into storage as policy revision 1. For later revisions, authority tooling
///      diffs the previous and new compiled ID sets and calls `applyPolicyUpdate` with
///      the status delta and the hash of the new complete document.
///
///      The registry does not verify that a status delta corresponds to a policy
///      hash; the authorized caller attests to that relationship. The status mapping
///      is the sole source of admission decisions. The hashes, revision, and count
///      support provenance, update sequencing, auditing, and consistency checks.
///
///      This contract is installed as a genesis predeploy, so its constructor is
///      deliberately empty and its initial state must be written directly into
///      genesis storage.
/// @custom:spec https://github.com/SeismicSystems/enclave/blob/seismic/crates/measurement-admission/SPEC.md
contract MeasurementRegistry {
    enum Status {
        Unknown,
        Accepted,
        Deprecated
    }

    /// @custom:storage-location erc7201:seismic.storage.MeasurementRegistry
    struct RegistryStorage {
        // Authoritative admission status for each compiled admission ID.
        // This is the only field consulted by nodes making an admission decision.
        // All the other fields are for humans to audit only.
        mapping(bytes32 admissionId => Status status) statuses;
        // Audit commitment to the exact bootstrap policy document compiled into
        // genesis revision 1. Written directly into genesis and never changed.
        bytes32 bootstrapPolicyHash;
        // Audit commitment to the exact complete policy document for the current
        // revision. Initially equal to bootstrapPolicyHash. Update validation
        // compares this value, but admission decisions do not consult it.
        bytes32 activePolicyHash;
        // Monotonically increasing policy revision used for initialization and
        // update sequencing. Initialized to 1 in genesis; 0 means uninitialized.
        uint64 policyRevision;
        // Cached number of concrete admission clauses currently Accepted after
        // policy expansion. Used for auditing and consistency, not admission.
        uint256 acceptedCount;
    }

    /// @dev keccak256(abi.encode(uint256(
    ///          keccak256("seismic.storage.MeasurementRegistry")) - 1))
    ///      & ~bytes32(uint256(0xff)). Hardcoded to avoid runtime derivation;
    ///      its value is pinned against this expression by a golden test.
    bytes32 internal constant REGISTRY_STORAGE_LOCATION =
        0xa3ae60943e4f183142036d77b94858085814dd428f131289aea7e42703fb0b00;

    /// @notice Well-known address of the contract authorized to mutate policy.
    address public constant AUTHORITY = 0x1000000000000000000000000000000000000002;

    error Unauthorized(address caller);
    error Uninitialized();
    error EmptyUpdate();
    error UnchangedPolicyHash(bytes32 policyHash);
    error DuplicateAdmissionId(bytes32 admissionId);
    error ContradictoryAdmissionId(bytes32 admissionId);
    error InvalidTransition(bytes32 admissionId, Status currentStatus, Status requestedStatus);

    /// @notice Emitted for each admission ID changed by a policy update.
    /// @param admissionId The concrete admission clause whose status changed.
    /// @param previousStatus Status before the update.
    /// @param newStatus Status after the update.
    /// @param policyRevision The revision resulting from the update.
    event AdmissionStatusChanged(
        bytes32 indexed admissionId, Status previousStatus, Status newStatus, uint64 indexed policyRevision
    );

    /// @notice Emitted after all per-ID status changes for a policy update.
    /// @param policyRevision The revision resulting from the update.
    /// @param activePolicyHash SHA-256 of the exact complete policy document for this revision.
    /// @param accepted Number of IDs accepted or reinstated by this update, not the current total.
    /// @param deprecated Number of IDs deprecated by this update, not the current total.
    event PolicyUpdated(
        uint64 indexed policyRevision, bytes32 indexed activePolicyHash, uint256 accepted, uint256 deprecated
    );

    modifier onlyAuthority() {
        if (msg.sender != AUTHORITY) revert Unauthorized(msg.sender);
        _;
    }

    /// @notice Returns the authoritative admission decision for one compiled ID.
    function isAccepted(bytes32 admissionId) external view returns (bool) {
        return _getRegistryStorage().statuses[admissionId] == Status.Accepted;
    }

    /// @notice Returns the complete status of `admissionId`.
    function statusOf(bytes32 admissionId) external view returns (Status) {
        return _getRegistryStorage().statuses[admissionId];
    }

    /// @notice SHA-256 of the exact bootstrap policy document compiled into genesis revision 1.
    /// @dev This is an immutable audit commitment and is not consulted by `isAccepted`.
    function bootstrapPolicyHash() external view returns (bytes32) {
        return _getRegistryStorage().bootstrapPolicyHash;
    }

    /// @notice SHA-256 of the exact complete policy document for the current revision.
    /// @dev This is an audit commitment and is not consulted by `isAccepted`.
    function activePolicyHash() external view returns (bytes32) {
        return _getRegistryStorage().activePolicyHash;
    }

    /// @notice Current policy revision. A valid genesis registry starts at revision 1.
    function policyRevision() external view returns (uint64) {
        return _getRegistryStorage().policyRevision;
    }

    /// @notice Number of concrete admission clauses currently in the Accepted state.
    /// @dev This is a cached audit and consistency value; it is not consulted by `isAccepted`.
    function acceptedCount() external view returns (uint256) {
        return _getRegistryStorage().acceptedCount;
    }

    /// @notice Applies the admission-ID difference for a new complete policy document.
    /// @dev Authority tooling is expected to compile the previous and new complete
    ///      policy documents, set `accept` to IDs present only in the new set, and set
    ///      `deprecate` to IDs present only in the previous set.
    ///
    ///      `newActivePolicyHash` identifies the complete new policy document, not a delta.
    ///      The registry trusts the authority to provide a matching ID delta and hash.
    ///      All status changes, metadata changes, and events are atomic.
    ///
    ///      Genesis must initialize the registry at revision 1. This function cannot
    ///      be used as a post-genesis initialization path.
    /// @param accept IDs transitioning from Unknown or Deprecated to Accepted.
    /// @param deprecate IDs transitioning from Accepted to Deprecated.
    /// @param newActivePolicyHash SHA-256 of the exact complete new policy document bytes.
    function applyPolicyUpdate(bytes32[] calldata accept, bytes32[] calldata deprecate, bytes32 newActivePolicyHash)
        external
        onlyAuthority
    {
        RegistryStorage storage registry = _getRegistryStorage();

        if (registry.policyRevision == 0) revert Uninitialized();
        if (accept.length == 0 && deprecate.length == 0) revert EmptyUpdate();
        if (newActivePolicyHash == registry.activePolicyHash) {
            revert UnchangedPolicyHash(newActivePolicyHash);
        }

        _validateBatch(registry, accept, deprecate);

        uint64 newPolicyRevision = registry.policyRevision + 1;
        uint256 newAcceptedCount = registry.acceptedCount;

        for (uint256 i; i < accept.length; ++i) {
            bytes32 admissionId = accept[i];
            Status previousStatus = registry.statuses[admissionId];
            registry.statuses[admissionId] = Status.Accepted;
            ++newAcceptedCount;
            emit AdmissionStatusChanged(admissionId, previousStatus, Status.Accepted, newPolicyRevision);
        }

        for (uint256 i; i < deprecate.length; ++i) {
            bytes32 admissionId = deprecate[i];
            registry.statuses[admissionId] = Status.Deprecated;
            --newAcceptedCount;
            emit AdmissionStatusChanged(admissionId, Status.Accepted, Status.Deprecated, newPolicyRevision);
        }

        registry.activePolicyHash = newActivePolicyHash;
        registry.policyRevision = newPolicyRevision;
        registry.acceptedCount = newAcceptedCount;

        emit PolicyUpdated(newPolicyRevision, newActivePolicyHash, accept.length, deprecate.length);
    }

    function _validateBatch(RegistryStorage storage registry, bytes32[] calldata accept, bytes32[] calldata deprecate)
        private
        view
    {
        for (uint256 i; i < accept.length; ++i) {
            bytes32 admissionId = accept[i];

            for (uint256 j; j < i; ++j) {
                if (accept[j] == admissionId) revert DuplicateAdmissionId(admissionId);
            }
            for (uint256 j; j < deprecate.length; ++j) {
                if (deprecate[j] == admissionId) {
                    revert ContradictoryAdmissionId(admissionId);
                }
            }

            Status currentStatus = registry.statuses[admissionId];
            if (currentStatus == Status.Accepted) {
                revert InvalidTransition(admissionId, currentStatus, Status.Accepted);
            }
        }

        for (uint256 i; i < deprecate.length; ++i) {
            bytes32 admissionId = deprecate[i];

            for (uint256 j; j < i; ++j) {
                if (deprecate[j] == admissionId) {
                    revert DuplicateAdmissionId(admissionId);
                }
            }

            Status currentStatus = registry.statuses[admissionId];
            if (currentStatus != Status.Accepted) {
                revert InvalidTransition(admissionId, currentStatus, Status.Deprecated);
            }
        }
    }

    function _getRegistryStorage() private pure returns (RegistryStorage storage registry) {
        bytes32 location = REGISTRY_STORAGE_LOCATION;
        assembly {
            registry.slot := location
        }
    }
}
