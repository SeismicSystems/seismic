"""ABI remapping for Seismic shielded types.

Seismic contracts use shielded types (``suint256``, ``sbool``, etc.)
that compile to ``CLOAD``/``CSTORE`` instead of ``SLOAD``/``SSTORE``.
These types need to be remapped for ABI encoding:

- The **function selector** uses the original ABI signature (with
  ``suint``, ``sbool``, etc.) to match the on-chain contract.
- The **parameter encoding** uses standard Solidity types (``uint``,
  ``bool``, etc.) because the values are structurally identical.

This module provides utilities to perform this remapping.
"""

from __future__ import annotations

import re
from copy import deepcopy
from typing import Any

from eth_abi import decode, encode
from eth_hash.auto import keccak
from hexbytes import HexBytes


def _remap_type(solidity_type: str) -> tuple[str, bool]:
    """Remap a single Seismic shielded type to its standard equivalent.

    Args:
        solidity_type: A Solidity type string, possibly shielded.

    Returns:
        Tuple of (remapped_type, was_shielded).
    """
    # suint/sint with fixed-size array: suint256[5] → uint256[5]
    m = re.match(r"^(s)(u?int\d+)(\[\d+\])$", solidity_type)
    if m:
        return f"{m.group(2)}{m.group(3)}", True

    # suint/sint with dynamic array: suint256[] → uint256[]
    m = re.match(r"^(s)(u?int\d+)(\[\])$", solidity_type)
    if m:
        return f"{m.group(2)}{m.group(3)}", True

    # suint/sint scalar: suint256 → uint256
    m = re.match(r"^s(u?int\d+)$", solidity_type)
    if m:
        return m.group(1), True

    # sbool / sbool[]
    if solidity_type == "sbool":
        return "bool", True
    if solidity_type == "sbool[]":
        return "bool[]", True

    # saddress / saddress[]
    if solidity_type == "saddress":
        return "address", True
    if solidity_type == "saddress[]":
        return "address[]", True

    return solidity_type, False


def remap_seismic_param(param: dict[str, Any]) -> dict[str, Any]:
    """Remap one ABI parameter from shielded to standard type.

    Handles ``suintN``, ``sintN``, ``sbool``, ``saddress`` — including
    array variants and recursive tuple components.

    Args:
        param: An ABI parameter dict (``{"name": ..., "type": ...}``).

    Returns:
        A new dict with the ``type`` remapped and a ``shielded`` flag added.
    """
    result = deepcopy(param)
    ty = result["type"]

    # Recursive tuple handling
    if ty.startswith("tuple"):
        components = result.get("components", [])
        mapped = [remap_seismic_param(c) for c in components]
        result["components"] = mapped
        result["shielded"] = any(c.get("shielded", False) for c in mapped)
        return result

    remapped, shielded = _remap_type(ty)
    result["type"] = remapped
    result["shielded"] = shielded
    return result


def remap_abi_inputs(abi_function: dict[str, Any]) -> dict[str, Any]:
    """Remap all inputs of an ABI function entry.

    Outputs are left unchanged (shielded types only affect inputs).

    Args:
        abi_function: An ABI function entry dict.

    Returns:
        A new dict with remapped input types.
    """
    result = deepcopy(abi_function)
    result["inputs"] = [remap_seismic_param(p) for p in result.get("inputs", [])]
    return result


def _abi_type_string(param: dict[str, Any]) -> str:
    """Build the canonical ABI type string for a parameter.

    For tuples, recursively builds ``(type1,type2,...)``.
    """
    ty = param["type"]
    if ty.startswith("tuple"):
        components = param.get("components", [])
        inner = ",".join(_abi_type_string(c) for c in components)
        suffix = ty[5:]  # e.g. "[]" or "[3]" or ""
        return f"({inner}){suffix}"
    return ty


def _function_signature(abi_function: dict[str, Any]) -> str:
    """Build the canonical function signature string.

    Example: ``"setNumber(suint256)"``
    """
    name = abi_function["name"]
    params = ",".join(_abi_type_string(p) for p in abi_function.get("inputs", []))
    return f"{name}({params})"


def _function_selector(abi_function: dict[str, Any]) -> bytes:
    """Compute the 4-byte function selector from the original ABI entry.

    Uses the **original** (un-remapped) type names so the selector
    matches the on-chain contract.

    Args:
        abi_function: The original ABI function entry.

    Returns:
        4-byte selector.
    """
    sig = _function_signature(abi_function)
    return keccak(sig.encode())[:4]


def encode_shielded_calldata(
    abi: list[dict[str, Any]],
    function_name: str,
    args: list[Any],
) -> HexBytes:
    """Encode calldata for a shielded contract function.

    The selector is computed from the **original** ABI (with shielded
    type names like ``suint256``), while the parameters are encoded
    using **remapped** standard types (``uint256``).

    Args:
        abi: The full contract ABI (list of function entries).
        function_name: Name of the function to call.
        args: Positional arguments matching the function inputs.

    Returns:
        Encoded calldata (4-byte selector + ABI-encoded parameters).

    Raises:
        ValueError: If the function is not found in the ABI.
    """
    # Find the function in the ABI
    fn_entry = None
    for entry in abi:
        if entry.get("type") == "function" and entry.get("name") == function_name:
            fn_entry = entry
            break

    if fn_entry is None:
        raise ValueError(f"Function '{function_name}' not found in ABI")

    # Selector from ORIGINAL types
    selector = _function_selector(fn_entry)

    # Encode params with REMAPPED types
    remapped = remap_abi_inputs(fn_entry)
    param_types = [_abi_type_string(p) for p in remapped["inputs"]]

    encoded_params = encode(param_types, args) if param_types else b""

    return HexBytes(selector + encoded_params)


def decode_abi_output(
    abi: list[dict[str, Any]],
    function_name: str,
    data: bytes,
) -> Any:
    """Decode raw ABI-encoded output bytes for a contract function.

    Looks up the function in the ABI, reads its ``outputs``, and decodes
    the raw bytes using ``eth_abi.decode``.

    - **Single output**: returns the value directly (e.g. ``int``, ``bool``,
      ``str``).
    - **Multiple outputs**: returns a ``tuple`` of decoded values.
    - **No outputs defined**: returns empty ``HexBytes``.
    - **Empty data with outputs defined**: zero-pads and decodes (e.g.
      ``uint256`` → ``0``, ``bool`` → ``False``).

    Args:
        abi: The full contract ABI (list of function entries).
        function_name: Name of the function whose output to decode.
        data: Raw ABI-encoded output bytes.

    Returns:
        Decoded Python value(s), or empty ``HexBytes`` when the ABI
        defines no outputs.

    Raises:
        ValueError: If the function is not found in the ABI.
    """
    # Find the function in the ABI
    fn_entry = None
    for entry in abi:
        if entry.get("type") == "function" and entry.get("name") == function_name:
            fn_entry = entry
            break

    if fn_entry is None:
        raise ValueError(f"Function '{function_name}' not found in ABI")

    outputs = fn_entry.get("outputs", [])

    if not outputs:
        return HexBytes(b"")

    # Build type strings from output params (no shielded remapping needed
    # because shielded types only affect inputs/storage, not return values)
    output_types = [_abi_type_string(p) for p in outputs]

    # Empty data with outputs defined: zero-pad so eth_abi decodes to
    # default values (0 for uint, False for bool, etc.)
    if not data:
        data = b"\x00" * 32 * len(output_types)

    decoded = decode(output_types, data)

    if len(output_types) == 1:
        return decoded[0]

    return decoded
