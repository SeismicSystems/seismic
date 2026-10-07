"""Ciphertext admission bounds must apply to the final write, not its read twin."""

from unittest.mock import patch

import pytest
import rlp
from hexbytes import HexBytes

from seismic_web3 import GasPayment
from seismic_web3.transaction.gas import seismic_pool_gas_minimum
from seismic_web3.transaction.send import (
    _async_prepare_shielded_transaction,
    _prepare_shielded_transaction,
)
from tests.test_contract import _make_encryption
from tests.test_send import ANVIL_ADDRESS, ANVIL_PK, _mock_async_w3, _mock_w3


@pytest.mark.parametrize(
    ("data", "create", "authorizations", "expected"),
    [
        (b"", False, 0, 21_000),
        (b"\0" * 20, False, 0, 21_200),
        (b"\xff" * 20, False, 0, 21_800),
        (b"\xff" * 19 + b"\0", False, 0, 21_770),
        (b"", True, 0, 53_000),
        (b"\xff" * 32, True, 0, 53_514),
        (b"\xff" * 33, True, 0, 53_532),
        (b"\xff" * 2_000, True, 0, 101_000),
        (b"\xff" * 20, False, 2, 71_320),
    ],
)
def test_pool_minimum_matches_prague_rules(data, create, authorizations, expected):
    assert (
        seismic_pool_gas_minimum(
            data,
            is_create=create,
            authorization_count=authorizations,
        )
        == expected
    )


_CASES = [
    # The final ciphertext requires MORE gas than the estimation twin's bytes.
    (21_480, None, b"\xff" * 20, b"\0" * 20, 21_800),
    # Use the exact final bytes, not the twin or a worst-case length bound.
    (21_480, None, b"\xff" * 19 + b"\0", b"\xff" * 20, 21_770),
    (21_480, None, b"\0" * 20, b"\xff" * 20, 21_480),
    # Preserve an execution estimate above the admission minimum.
    (100_000, None, b"\xff" * 20, b"\0" * 20, 100_000),
    # Even a too-low explicit gas limit must be preserved, without estimation.
    (21_480, 21_000, b"\xff" * 20, b"\0" * 20, 21_000),
]
_PAYMENTS = [
    None,
    GasPayment.auto(),
    GasPayment.native(),
    GasPayment.token("0x" + "11" * 20),
]


def _assert_prepared(w3, signed, tx, write, twin, gas, expected, typed):
    assert tx.gas == expected
    assert tx.data == write
    fields = rlp.decode(bytes(signed[1:]))
    assert int.from_bytes(fields[3], "big") == expected
    assert fields[13] == write
    assert fields[12] == b""  # the real transaction remains broadcastable
    if gas is None:
        w3.provider.make_request.assert_called_once()
        method, params = w3.provider.make_request.call_args.args
        assert method == "eth_estimateGas"
        estimate = rlp.decode(bytes(HexBytes(params[0])[1:]))
        assert estimate[13] == twin
        assert estimate[12] == b"\x01"  # estimate remains non-broadcastable
    else:
        w3.provider.make_request.assert_not_called()
    assert tx.seismic.message_version == (2 if typed else 0)


@pytest.mark.parametrize("payment", _PAYMENTS)
@pytest.mark.parametrize("typed", [False, True])
@pytest.mark.parametrize(("estimate", "gas", "write", "twin", "expected"), _CASES)
def test_final_write_gas_is_clamped_before_signing(
    payment,
    typed,
    estimate,
    gas,
    write,
    twin,
    expected,
):
    w3 = _mock_w3(hex(estimate))
    w3.eth.get_block.return_value["gasLimit"] = 30_000_000
    encryption = _make_encryption()
    with patch.object(type(encryption), "encrypt", side_effect=[write, twin]):
        signed, tx, _ = _prepare_shielded_transaction(
            w3,
            encryption=encryption,
            private_key=ANVIL_PK,
            to=ANVIL_ADDRESS,
            data=HexBytes("0x313ce567"),
            gas=gas,
            gas_payment=payment,
            eip712=typed,
        )
    _assert_prepared(w3, signed, tx, write, twin, gas, expected, typed)
    assert tx.gas_payment == (payment or GasPayment.auto())


@pytest.mark.parametrize("payment", _PAYMENTS)
@pytest.mark.parametrize("typed", [False, True])
@pytest.mark.parametrize(("estimate", "gas", "write", "twin", "expected"), _CASES)
async def test_async_final_write_gas_is_clamped_before_signing(
    payment,
    typed,
    estimate,
    gas,
    write,
    twin,
    expected,
):
    w3 = _mock_async_w3(hex(estimate))
    w3.eth.get_block.return_value["gasLimit"] = 30_000_000
    encryption = _make_encryption()
    with patch.object(type(encryption), "encrypt", side_effect=[write, twin]):
        signed, tx, _ = await _async_prepare_shielded_transaction(
            w3,
            encryption=encryption,
            private_key=ANVIL_PK,
            to=ANVIL_ADDRESS,
            data=HexBytes("0x313ce567"),
            gas=gas,
            gas_payment=payment,
            eip712=typed,
        )
    _assert_prepared(w3, signed, tx, write, twin, gas, expected, typed)
    assert tx.gas_payment == (payment or GasPayment.auto())
