// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

/// @title TxUtils
/// @notice Helpers for reading transaction-level context on Seismic via the tx-info precompile.
/// @dev Uses a `staticcall` to the tx-info precompile (0x6A) rather than a custom opcode, so it
/// compiles with a stock Solidity compiler — no `ssolc` builtin required.
library TxUtils {
    /// @notice Address of the Seismic tx-info precompile.
    address internal constant TX_INFO_PRECOMPILE = address(0x6A);

    uint256 internal constant SEISMIC_TX_TYPE = 0x4A;

    /// @notice EIP-2718 transaction-type byte of the current transaction (74 = Seismic).
    function txType() internal view returns (uint256 t) {
        (bool ok, bytes memory ret) = TX_INFO_PRECOMPILE.staticcall("");
        require(ok && ret.length == 32, "TX_INFO");
        t = abi.decode(ret, (uint256));
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
