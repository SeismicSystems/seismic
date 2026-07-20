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

    function test_txType_isByteRange() public view {
        assertLe(harness.txType(), 255);
    }

    function test_isSeismicTx_matchesTxType() public view {
        assertEq(harness.isSeismicTx(), harness.txType() == 0x4A);
    }
}
