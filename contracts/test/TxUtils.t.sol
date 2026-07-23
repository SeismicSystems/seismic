// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import {TxUtils} from "../src/seismic-std-lib/utils/TxUtils.sol";

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

    function setUp() public {
        harness = new TxUtilsHarness();
    }

    // In the forge unit executor the call is a standard (non-Seismic) tx, so txtype()
    // must report 0 and isSeismicTx() false. This catches an opcode that wrongly returns
    // a constant (e.g. always 74). Real Seismic-tx / signed-read cases (txtype()==74) are
    // covered by the node integration tests (sanvil/sreth), which the unit harness cannot
    // reproduce (no cheatcode sets the EIP-2718 tx type).
    function test_txType_isZeroForStandardCall() public view {
        assertEq(harness.txType(), 0);
    }

    function test_isSeismicTx_falseForStandardCall() public view {
        assertFalse(harness.isSeismicTx());
    }
}
