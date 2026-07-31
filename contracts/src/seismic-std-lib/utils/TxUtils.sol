// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

/// @title TxUtils
/// @notice Helpers for reading transaction-level context on Seismic via the tx-type precompile.
/// @dev Uses a `staticcall` to the tx-type precompile (0x6A) rather than a custom opcode, so it
/// compiles with a stock Solidity compiler — no `ssolc` builtin required.
library TxUtils {
    /// @notice Address of the Seismic tx-type precompile.
    address internal constant TX_TYPE_PRECOMPILE = address(0x6A);

    uint256 internal constant SEISMIC_TX_TYPE = 0x4A;

    /// @notice Thrown when the tx-type precompile is unavailable (e.g. on a non-Seismic EVM).
    error TxTypePrecompileUnavailable();

    /// @notice EIP-2718 transaction-type byte of the current transaction (74 = Seismic).
    /// @dev **Fail-closed:** reverts with {TxTypePrecompileUnavailable} on an EVM that lacks the
    /// `0x6A` precompile — there the `staticcall` succeeds with empty returndata rather than a
    /// 32-byte type, so this never silently returns 0.
    function txType() internal view returns (uint256 t) {
        (bool ok, bytes memory ret) = TX_TYPE_PRECOMPILE.staticcall("");
        if (!ok || ret.length != 32) revert TxTypePrecompileUnavailable();
        t = abi.decode(ret, (uint256));
    }

    /// @notice True when executing in an authenticated Seismic context: a real Seismic
    /// transaction (type 0x4A) or an authenticated signed read. False for a plain,
    /// unauthenticated `eth_call`.
    /// @dev Reports the transaction type only. This is NOT an authorization check, does not
    /// prove the call is state-changing (authenticated signed reads also return true), and does
    /// not by itself guarantee that every field of the RPC response is encrypted. **Reverts** (it
    /// does not return `false`) on an EVM without the precompile — see {txType}.
    function isSeismicTx() internal view returns (bool) {
        return txType() == SEISMIC_TX_TYPE;
    }
}
