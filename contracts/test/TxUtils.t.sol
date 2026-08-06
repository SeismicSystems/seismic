// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import {TxUtils} from "../src/seismic-std-lib/utils/TxUtils.sol";

/// Minimal view of the Seismic tx-context cheatcodes (set the EIP-2718 tx type and the signed-read
/// flag for subsequent calls), until forge-std's Vm interface is regenerated with them.
interface VmSeismicCtx {
    function txType(uint8 newTxType) external;
    function signedRead(bool isSignedRead) external;
}

contract TxUtilsHarness {
    function txType() external view returns (uint256) {
        return TxUtils.txType();
    }

    function isSeismicTx() external view returns (bool) {
        return TxUtils.isSeismicTx();
    }

    function isSignedRead() external view returns (bool) {
        return TxUtils.isSignedRead();
    }
}

contract TxUtilsTest is Test {
    TxUtilsHarness internal harness;
    VmSeismicCtx internal vmCtx;

    function setUp() public {
        harness = new TxUtilsHarness();
        vmCtx = VmSeismicCtx(address(vm));
    }

    function test_txType_defaultsToStandardCall() public view {
        assertEq(harness.txType(), 0);
        assertFalse(harness.isSeismicTx());
        assertFalse(harness.isSignedRead());
    }

    function test_isSeismicTx_trueForSeismicTxType() public {
        vmCtx.txType(0x4A);
        assertEq(harness.txType(), 0x4A);
        assertTrue(harness.isSeismicTx());
    }

    function test_isSeismicTx_falseForStandardTxType() public {
        vmCtx.txType(2);
        assertEq(harness.txType(), 2);
        assertFalse(harness.isSeismicTx());
    }

    /// A signed read: Seismic-typed and flagged as a read. Both predicates are true.
    function test_isSignedRead_trueForSignedRead() public {
        vmCtx.txType(0x4A);
        vmCtx.signedRead(true);
        assertTrue(harness.isSignedRead());
        assertTrue(harness.isSeismicTx(), "a signed read is a Seismic context");
    }

    /// A mined Seismic write: Seismic-typed but not a read. isSeismicTx true, isSignedRead false.
    function test_isSignedRead_falseForSeismicWrite() public {
        vmCtx.txType(0x4A);
        vmCtx.signedRead(false);
        assertFalse(harness.isSignedRead());
        assertTrue(harness.isSeismicTx());
    }

    function test_isSignedRead_defaultsFalse() public view {
        assertFalse(harness.isSignedRead());
    }
}
