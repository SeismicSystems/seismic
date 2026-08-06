// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

/// @title TxUtils
/// @notice Reads Seismic transaction context via the 0x6A tx-context precompile (stock Solidity, no
/// ssolc builtin). Empty input selects the tx type; a 0x01 byte selects the signed-read flag.
library TxUtils {
    address internal constant TX_CONTEXT_PRECOMPILE = address(0x6A);
    uint256 internal constant SEISMIC_TX_TYPE = 0x4A;
    bytes internal constant SIGNED_READ_SELECTOR = hex"01";

    /// @notice Thrown when the 0x6A precompile is unavailable (e.g. on a non-Seismic EVM).
    error TxContextPrecompileUnavailable();

    // Fail-closed: an EVM without 0x6A returns empty data, so a caller reverts rather than reads 0.
    function _read(bytes memory selector) private view returns (uint256 v) {
        (bool ok, bytes memory ret) = TX_CONTEXT_PRECOMPILE.staticcall(selector);
        if (!ok || ret.length != 32) revert TxContextPrecompileUnavailable();
        v = abi.decode(ret, (uint256));
    }

    /// @notice EIP-2718 transaction-type byte of the current transaction (74 = Seismic).
    function txType() internal view returns (uint256) {
        return _read("");
    }

    /// @notice True in an authenticated Seismic context (a Seismic tx or an authenticated signed
    /// read). Reports tx type only: not an authorization or confidentiality guarantee. Reverts on an
    /// EVM without the precompile.
    function isSeismicTx() internal view returns (bool) {
        return txType() == SEISMIC_TX_TYPE;
    }

    /// @notice True when the current execution is an authenticated signed read (an RPC read) rather
    /// than a mined tx; implies {isSeismicTx}.
    /// @dev The node's RPC classification, NOT an EVM guarantee of no state change — a signed read
    /// can still SSTORE (the node discards it). A context signal, like {isSeismicTx}.
    function isSignedRead() internal view returns (bool) {
        return _read(SIGNED_READ_SELECTOR) == 1;
    }
}
