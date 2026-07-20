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

    /// @notice True if the current transaction is a Seismic (type 0x4A) transaction.
    function isSeismicTx() internal view returns (bool) {
        return txType() == SEISMIC_TX_TYPE;
    }
}
