// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import {TxUtils} from "../src/seismic-std-lib/utils/TxUtils.sol";

/// Minimal view of the Seismic `txType` cheatcode (sets the EIP-2718 tx type for
/// subsequent calls), until forge-std's Vm interface is regenerated with it.
interface VmTxType {
    function txType(uint8 newTxType) external;
}

contract TxUtilsHarness {
    function txType() external view returns (uint256) {
        return TxUtils.txType();
    }

    function isSeismicTx() external view returns (bool) {
        return TxUtils.isSeismicTx();
    }
}

contract TxUtilsTest is Test {
    TxUtilsHarness internal harness;
    VmTxType internal vmTx;

    function setUp() public {
        harness = new TxUtilsHarness();
        vmTx = VmTxType(address(vm));
    }

    function test_txType_defaultsToStandardCall() public view {
        assertEq(harness.txType(), 0);
        assertFalse(harness.isSeismicTx());
    }

    function test_isSeismicTx_trueForSeismicTxType() public {
        vmTx.txType(0x4A);
        assertEq(harness.txType(), 0x4A);
        assertTrue(harness.isSeismicTx());
    }

    function test_isSeismicTx_falseForStandardTxType() public {
        vmTx.txType(2);
        assertEq(harness.txType(), 2);
        assertFalse(harness.isSeismicTx());
    }
}
