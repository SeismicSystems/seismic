"""Tests for seismic_web3.contract.abi — ABI remapping for shielded types."""

import pytest
from eth_abi import encode
from eth_hash.auto import keccak

from seismic_web3.contract.abi import (
    decode_abi_output,
    encode_shielded_calldata,
    has_shielded_params,
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


FULL_COUNTER_ABI = [
    *COUNTER_ABI,
    {
        "type": "function",
        "name": "isOdd",
        "inputs": [],
        "outputs": [{"name": "", "type": "bool"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "getNumber",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view",
    },
]


class TestHasShieldedParams:
    def test_shielded_function(self):
        """setNumber(suint256) has shielded params."""
        assert has_shielded_params(COUNTER_ABI, "setNumber") is True

    def test_transparent_function(self):
        """increment() has no shielded params."""
        assert has_shielded_params(COUNTER_ABI, "increment") is False

    def test_function_not_found(self):
        with pytest.raises(ValueError, match="not found"):
            has_shielded_params(COUNTER_ABI, "nonexistent")

    def test_view_no_inputs(self):
        """isOdd() with no inputs is not shielded."""
        assert has_shielded_params(FULL_COUNTER_ABI, "isOdd") is False

    def test_view_no_inputs_getNumber(self):
        """getNumber() with no inputs is not shielded."""
        assert has_shielded_params(FULL_COUNTER_ABI, "getNumber") is False


class TestEncodeShieldedCalldata:
    @pytest.mark.parametrize("reverse", [False, True])
    @pytest.mark.parametrize(
        ("param_type", "value"),
        [
            ("uint256", 42),
            ("address", "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
        ],
    )
    def test_overload_selected_by_arguments(self, reverse, param_type, value):
        abi = [
            {
                "type": "function",
                "name": "lookup",
                "inputs": [{"type": ty}],
            }
            for ty in ("uint256", "address")
        ]
        if reverse:
            abi.reverse()

        calldata = encode_shielded_calldata(abi, "lookup", [value])

        expected = keccak(f"lookup({param_type})".encode())[:4]
        assert bytes(calldata) == expected + encode([param_type], [value])

    @pytest.mark.parametrize("args", [[], [42], [42, 7]])
    def test_overload_selected_by_argument_count(self, args):
        abi = [
            {
                "type": "function",
                "name": "lookup",
                "inputs": [{"type": "uint256"}] * count,
            }
            for count in (2, 1, 0)
        ]
        types = ["uint256"] * len(args)
        expected = keccak(f"lookup({','.join(types)})".encode())[:4]

        assert bytes(encode_shielded_calldata(abi, "lookup", args)) == (
            expected + encode(types, args)
        )

    def test_overload_matches_remapped_tuple_array(self):
        abi = [
            {
                "type": "function",
                "name": "lookup",
                "inputs": [{"type": "saddress"}],
            },
            {
                "type": "function",
                "name": "lookup",
                "inputs": [
                    {
                        "type": "tuple[]",
                        "components": [{"type": "suint256"}, {"type": "sbool"}],
                    },
                ],
            },
        ]
        args = [[(42, True)]]
        expected = keccak(b"lookup((suint256,sbool)[])")[:4]

        assert bytes(encode_shielded_calldata(abi, "lookup", args)) == (
            expected + encode(["(uint256,bool)[]"], args)
        )

    @pytest.mark.parametrize("args", [[], [True], [42, 7]])
    def test_no_matching_overload_raises(self, args):
        abi = [
            {
                "type": "function",
                "name": "lookup",
                "inputs": [{"type": ty}],
            }
            for ty in ("uint256", "address")
        ]
        with pytest.raises(ValueError, match="No matching overload"):
            encode_shielded_calldata(abi, "lookup", args)

    @pytest.mark.parametrize("types", [("uint8", "uint256"), ("uint256", "suint256")])
    @pytest.mark.parametrize("reverse", [False, True])
    def test_ambiguous_overload_raises(self, types, reverse):
        abi = [
            {
                "type": "function",
                "name": "lookup",
                "inputs": [{"type": ty}],
            }
            for ty in types
        ]
        if reverse:
            abi.reverse()

        with pytest.raises(ValueError, match="Ambiguous overload"):
            encode_shielded_calldata(abi, "lookup", [42])

    def test_selector_uses_original_types(self):
        """The 4-byte selector must be computed from "setNumber(suint256)",
        not "setNumber(uint256)"."""
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


# ---------------------------------------------------------------------------
# decode_abi_output
# ---------------------------------------------------------------------------

DECODE_ABI = [
    {
        "type": "function",
        "name": "getNumber",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "isOdd",
        "inputs": [],
        "outputs": [{"name": "", "type": "bool"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "getMultiple",
        "inputs": [],
        "outputs": [
            {"name": "a", "type": "uint256"},
            {"name": "b", "type": "bool"},
            {"name": "c", "type": "address"},
        ],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "noOutputs",
        "inputs": [],
        "outputs": [],
        "stateMutability": "nonpayable",
    },
    {
        "type": "function",
        "name": "getName",
        "inputs": [],
        "outputs": [{"name": "", "type": "string"}],
        "stateMutability": "view",
    },
]


class TestDecodeAbiOutput:
    def test_single_uint256(self):
        raw = encode(["uint256"], [42])
        result = decode_abi_output(DECODE_ABI, "getNumber", raw)
        assert result == 42

    def test_single_bool_true(self):
        raw = encode(["bool"], [True])
        result = decode_abi_output(DECODE_ABI, "isOdd", raw)
        assert result is True

    def test_single_bool_false(self):
        raw = encode(["bool"], [False])
        result = decode_abi_output(DECODE_ABI, "isOdd", raw)
        assert result is False

    def test_multiple_outputs(self):
        addr = "0x000000000000000000000000000000000000dEaD"
        raw = encode(["uint256", "bool", "address"], [100, True, addr])
        result = decode_abi_output(DECODE_ABI, "getMultiple", raw)
        assert isinstance(result, tuple)
        assert result[0] == 100
        assert result[1] is True
        assert result[2].lower() == addr.lower()

    def test_no_outputs_defined(self):
        raw = encode(["uint256"], [1])
        result = decode_abi_output(DECODE_ABI, "noOutputs", raw)
        assert result is None

    def test_no_outputs_defined_empty_data(self):
        result = decode_abi_output(DECODE_ABI, "noOutputs", b"")
        assert result is None

    def test_empty_data_uint256(self):
        """Empty bytes for a uint256 output should decode to 0."""
        result = decode_abi_output(DECODE_ABI, "getNumber", b"")
        assert result == 0

    def test_empty_data_bool(self):
        """Empty bytes for a bool output should decode to False."""
        result = decode_abi_output(DECODE_ABI, "isOdd", b"")
        assert result is False

    def test_empty_data_multiple_outputs(self):
        """Empty bytes for multiple outputs should decode to zero values."""
        result = decode_abi_output(DECODE_ABI, "getMultiple", b"")
        assert isinstance(result, tuple)
        assert result[0] == 0
        assert result[1] is False
        assert result[2] == "0x" + "0" * 40

    def test_string_output(self):
        raw = encode(["string"], ["hello world"])
        result = decode_abi_output(DECODE_ABI, "getName", raw)
        assert result == "hello world"

    def test_function_not_found_raises(self):
        with pytest.raises(ValueError, match="not found"):
            decode_abi_output(DECODE_ABI, "nonexistent", b"\x00" * 32)
