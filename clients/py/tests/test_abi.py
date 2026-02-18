"""Tests for seismic_web3.contract.abi — ABI remapping for shielded types."""

import pytest

from seismic_web3.contract.abi import (
    encode_shielded_calldata,
    remap_abi_inputs,
    remap_seismic_param,
)

# ---------------------------------------------------------------------------
# remap_seismic_param
# ---------------------------------------------------------------------------


class TestRemapSeismicParam:
    def test_suint256(self):
        result = remap_seismic_param({"name": "x", "type": "suint256"})
        assert result["type"] == "uint256"
        assert result["shielded"] is True

    def test_sint128(self):
        result = remap_seismic_param({"name": "x", "type": "sint128"})
        assert result["type"] == "int128"
        assert result["shielded"] is True

    def test_sbool(self):
        result = remap_seismic_param({"name": "x", "type": "sbool"})
        assert result["type"] == "bool"
        assert result["shielded"] is True

    def test_saddress(self):
        result = remap_seismic_param({"name": "x", "type": "saddress"})
        assert result["type"] == "address"
        assert result["shielded"] is True

    def test_suint256_fixed_array(self):
        result = remap_seismic_param({"name": "x", "type": "suint256[5]"})
        assert result["type"] == "uint256[5]"
        assert result["shielded"] is True

    def test_sbool_dynamic_array(self):
        result = remap_seismic_param({"name": "x", "type": "sbool[]"})
        assert result["type"] == "bool[]"
        assert result["shielded"] is True

    def test_saddress_dynamic_array(self):
        result = remap_seismic_param({"name": "x", "type": "saddress[]"})
        assert result["type"] == "address[]"
        assert result["shielded"] is True

    def test_sint64_dynamic_array(self):
        result = remap_seismic_param({"name": "x", "type": "sint64[]"})
        assert result["type"] == "int64[]"
        assert result["shielded"] is True

    def test_non_shielded_passthrough(self):
        result = remap_seismic_param({"name": "x", "type": "uint256"})
        assert result["type"] == "uint256"
        assert result["shielded"] is False

    def test_address_passthrough(self):
        result = remap_seismic_param({"name": "x", "type": "address"})
        assert result["type"] == "address"
        assert result["shielded"] is False

    def test_tuple_recursive(self):
        param = {
            "name": "s",
            "type": "tuple",
            "components": [
                {"name": "a", "type": "suint256"},
                {"name": "b", "type": "uint256"},
            ],
        }
        result = remap_seismic_param(param)
        assert result["type"] == "tuple"
        assert result["shielded"] is True
        assert result["components"][0]["type"] == "uint256"
        assert result["components"][0]["shielded"] is True
        assert result["components"][1]["type"] == "uint256"
        assert result["components"][1]["shielded"] is False


# ---------------------------------------------------------------------------
# remap_abi_inputs
# ---------------------------------------------------------------------------


class TestRemapAbiInputs:
    def test_remaps_inputs_only(self):
        fn = {
            "type": "function",
            "name": "setNumber",
            "inputs": [{"name": "x", "type": "suint256"}],
            "outputs": [{"name": "", "type": "suint256"}],
            "stateMutability": "nonpayable",
        }
        result = remap_abi_inputs(fn)
        assert result["inputs"][0]["type"] == "uint256"
        # Outputs should remain unchanged
        assert result["outputs"][0]["type"] == "suint256"


# ---------------------------------------------------------------------------
# encode_shielded_calldata
# ---------------------------------------------------------------------------


COUNTER_ABI = [
    {
        "type": "function",
        "name": "setNumber",
        "inputs": [{"name": "newNumber", "type": "suint256"}],
        "outputs": [],
        "stateMutability": "nonpayable",
    },
    {
        "type": "function",
        "name": "increment",
        "inputs": [],
        "outputs": [],
        "stateMutability": "nonpayable",
    },
]


class TestEncodeShieldedCalldata:
    def test_selector_uses_original_types(self):
        """The 4-byte selector must be computed from "setNumber(suint256)",
        not "setNumber(uint256)"."""
        from eth_hash.auto import keccak

        calldata = encode_shielded_calldata(COUNTER_ABI, "setNumber", [42])

        expected_selector = keccak(b"setNumber(suint256)")[:4]
        assert bytes(calldata[:4]) == expected_selector

    def test_params_encoded_with_remapped_types(self):
        calldata = encode_shielded_calldata(COUNTER_ABI, "setNumber", [42])
        # After the 4-byte selector, the rest is ABI-encoded uint256(42)
        param_data = bytes(calldata[4:])
        assert len(param_data) == 32
        assert int.from_bytes(param_data, "big") == 42

    def test_no_args_function(self):
        calldata = encode_shielded_calldata(COUNTER_ABI, "increment", [])
        # Should be just the 4-byte selector
        assert len(calldata) == 4

    def test_function_not_found_raises(self):
        with pytest.raises(ValueError, match="not found"):
            encode_shielded_calldata(COUNTER_ABI, "nonexistent", [])
