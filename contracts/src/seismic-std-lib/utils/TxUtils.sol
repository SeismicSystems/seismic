// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

/// @title TxUtils
/// @notice Helpers for reading transaction-level context on Seismic.
library TxUtils {
    uint256 internal constant SEISMIC_TX_TYPE = 0x4A;

    /// @notice EIP-2718 transaction-type byte of the current transaction (74 = Seismic).
    function txType() internal view returns (uint256 t) {
        assembly {
            t := txtype()
        }
    }

    /// @notice True when executing in an authenticated Seismic context: a real Seismic
    /// transaction (type 0x4A) or an authenticated signed read. False for a plain,
    /// unauthenticated `eth_call`.
    /// @dev Reports the transaction type only. This is NOT an authorization check, does not
    /// prove the call is state-changing (authenticated signed reads also return true), and does
    /// not by itself guarantee that every field of the RPC response is encrypted.
    function isSeismicTx() internal view returns (bool) {
        return txType() == SEISMIC_TX_TYPE;
    }
}
