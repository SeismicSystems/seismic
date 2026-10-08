"""Mandatory selector parity and propagation through sync/async SDK paths."""

import json
from dataclasses import FrozenInstanceError, replace
from pathlib import Path
from typing import cast
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
import rlp
from eth_hash.auto import keccak
from eth_keys import keys
from eth_utils import to_checksum_address
from hexbytes import HexBytes

from seismic_web3 import GasPayment
from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)
from seismic_web3.contract.shielded import AsyncShieldedContract, ShieldedContract
from seismic_web3.gas_payment import require_auto_payment, resolve_gas_payment
from seismic_web3.module import AsyncSeismicNamespace, SeismicNamespace
from seismic_web3.transaction.aead import encode_metadata_as_aad
from seismic_web3.transaction.eip712 import eip712_signing_hash, sign_seismic_tx_eip712
from seismic_web3.transaction.send import (
    _async_prepare_shielded_transaction,
    _prepare_shielded_transaction,
)
from seismic_web3.transaction.serialize import (
    hash_unsigned,
    serialize_signed,
    serialize_unsigned,
    sign_seismic_tx,
)
from seismic_web3.transaction_types import (
    LegacyFields,
    SeismicElements,
    Signature,
    SignedAuthorization,
    TxSeismicMetadata,
    UnsignedSeismicTx,
)
from tests.test_contract import COUNTER_ABI, _make_encryption
from tests.test_send import ANVIL_PK, _mock_async_w3, _mock_w3

_FIXTURE = json.loads(
    (Path(__file__).parents[2] / "test-vectors" / "gas-payment.json").read_text(),
)
_ADDRESS = to_checksum_address("0x" + "33" * 20)
_PAYMENTS = [
    None,
    GasPayment.auto(),
    GasPayment.native(),
    GasPayment.token("0x" + "11" * 20),
]


def _vector_tx(vector):
    tx = vector["tx"]
    choice = tx["gasPayment"]
    payment = (
        GasPayment.token(choice["token"])
        if choice["type"] == "token"
        else GasPayment(choice["type"])
    )
    return UnsignedSeismicTx(
        chain_id=int(tx["chainId"], 16),
        nonce=int(tx["nonce"], 16),
        gas_price=int(tx["gasPrice"], 16),
        gas=int(tx["gas"], 16),
        gas_payment=payment,
        to=tx.get("to"),
        value=int(tx["value"], 16),
        data=HexBytes(tx["input"]),
        seismic=SeismicElements(
            encryption_pubkey=CompressedPublicKey(tx["encryptionPubkey"]),
            encryption_nonce=EncryptionNonce(
                int(tx["encryptionNonce"], 16).to_bytes(12, "big"),
            ),
            message_version=int(tx["messageVersion"], 16),
            recent_block_hash=Bytes32(tx["recentBlockHash"]),
            expires_at_block=int(tx["expiresAtBlock"], 16),
            signed_read=tx["signedRead"],
        ),
        authorization_list=[
            SignedAuthorization(
                chain_id=int(auth["chainId"], 16),
                address=auth["address"],
                nonce=int(auth["nonce"], 16),
                y_parity=int(auth["yParity"], 16),
                r=int(auth["r"], 16),
                s=int(auth["s"], 16),
            )
            for auth in tx["authorizationList"]
        ],
    )


@pytest.mark.parametrize("vector", _FIXTURE["vectors"])
def test_rust_golden_vectors(vector):
    tx = _vector_tx(vector)
    assert (b"\x4a" + serialize_unsigned(tx)).hex() == vector["unsigned"][2:]
    digest = (
        eip712_signing_hash(tx)
        if tx.seismic.message_version == 2
        else hash_unsigned(tx)
    )
    assert digest.hex() == vector["signingHash"][2:]
    key = PrivateKey(_FIXTURE["privateKey"])
    signed = (
        sign_seismic_tx_eip712(tx, key)
        if tx.seismic.message_version == 2
        else sign_seismic_tx(tx, key)
    )
    assert signed.to_0x_hex() == vector["signed"]
    assert keccak(signed).hex() == vector["txHash"][2:]
    fields = rlp.decode(bytes(signed[1:]))
    assert fields[4] == tx.gas_payment.rlp_parts()
    assert fields[8] == bytes(tx.seismic.encryption_nonce).lstrip(b"\0")
    other = GasPayment.native() if tx.gas_payment.kind == "auto" else GasPayment.auto()
    tampered = replace(tx, gas_payment=other)
    tampered_digest = (
        eip712_signing_hash(tampered)
        if tx.seismic.message_version == 2
        else hash_unsigned(tampered)
    )
    sig = vector["signature"]
    signature = Signature(
        v=int(sig["yParity"], 16),
        r=int(sig["r"], 16),
        s=int(sig["s"], 16),
    )
    assert serialize_signed(tx, signature) == signed
    recovered = keys.Signature(
        vrs=(signature.v, signature.r, signature.s),
    ).recover_public_key_from_msg_hash(tampered_digest)
    assert (
        recovered.to_checksum_address()
        != keys.PrivateKey(bytes(key)).public_key.to_checksum_address()
    )
    # Fee selection is outside the eleven-field AAD, whose nonce stays 12 bytes.
    metadata = TxSeismicMetadata(
        sender=keys.PrivateKey(bytes(key)).public_key.to_checksum_address(),
        legacy_fields=LegacyFields(tx.chain_id, tx.nonce, tx.to, tx.value),
        seismic_elements=tx.seismic,
    )
    assert rlp.decode(encode_metadata_as_aad(metadata))[6] == bytes(
        tx.seismic.encryption_nonce,
    )
    assert len(rlp.decode(encode_metadata_as_aad(metadata))) == 11


@pytest.mark.parametrize(
    ("kind", "address"),
    [
        ("unknown", None),
        ("auto", "0x" + "11" * 20),
        ("native", "0x" + "11" * 20),
        ("token", None),
        ("token", "0x" + "00" * 20),
        ("token", "0x01"),
    ],
)
def test_reject_malformed_choices(kind, address):
    with pytest.raises(ValueError, match="gas payment"):
        GasPayment(kind, address)


def test_defaults_validation_and_immutability():
    assert resolve_gas_payment() == GasPayment.auto()
    assert GasPayment.auto().to_json() == {"type": "auto"}
    assert GasPayment.native().rlp_parts() == [b"\x01", b""]
    assert GasPayment.token("0x" + "11" * 20).to_json()["token"] == "0x" + "11" * 20
    field_name = "kind"
    with pytest.raises(FrozenInstanceError):
        setattr(GasPayment.auto(), field_name, "native")
    with pytest.raises(TypeError):
        resolve_gas_payment(cast("GasPayment", {"type": "auto"}))
    require_auto_payment()
    require_auto_payment(GasPayment.auto())
    for payment in _PAYMENTS[2:]:
        with pytest.raises(ValueError, match="requires a Seismic"):
            require_auto_payment(payment)


@pytest.mark.parametrize("payment", _PAYMENTS)
@pytest.mark.parametrize("typed", [False, True])
def test_estimate_and_final_transaction_preserve_choice(payment, typed):
    w3 = _mock_w3("0x5208")
    w3.eth.get_block.return_value["gasLimit"] = 30_000_000
    signed, tx, _ = _prepare_shielded_transaction(
        w3,
        encryption=_make_encryption(),
        private_key=ANVIL_PK,
        to=_ADDRESS,
        data=HexBytes("0x1234"),
        gas_payment=payment,
        eip712=typed,
    )
    estimate = w3.provider.make_request.call_args.args[1][0]
    expected = resolve_gas_payment(payment)
    assert rlp.decode(bytes(HexBytes(estimate)[1:]))[4] == expected.rlp_parts()
    assert rlp.decode(bytes(signed[1:]))[4] == expected.rlp_parts()
    assert tx.gas_payment == expected


@pytest.mark.parametrize("payment", _PAYMENTS)
@pytest.mark.parametrize("typed", [False, True])
async def test_async_estimate_and_final_preserve_choice(payment, typed):
    w3 = _mock_async_w3("0x5208")
    w3.eth.get_block.return_value["gasLimit"] = 30_000_000
    signed, tx, _ = await _async_prepare_shielded_transaction(
        w3,
        encryption=_make_encryption(),
        private_key=ANVIL_PK,
        to=_ADDRESS,
        data=HexBytes("0x1234"),
        gas_payment=payment,
        eip712=typed,
    )
    estimate = w3.provider.make_request.call_args.args[1][0]
    expected = resolve_gas_payment(payment)
    assert rlp.decode(bytes(HexBytes(estimate)[1:]))[4] == expected.rlp_parts()
    assert rlp.decode(bytes(signed[1:]))[4] == expected.rlp_parts()
    assert tx.gas_payment == expected


@pytest.mark.parametrize("namespace", ["write", "swrite", "dwrite", "read", "sread"])
def test_contract_forwarding(namespace):
    contract = ShieldedContract(
        MagicMock(),
        _make_encryption(),
        ANVIL_PK,
        _ADDRESS,
        COUNTER_ABI,
    )
    helper = (
        "debug_send_shielded_transaction"
        if namespace == "dwrite"
        else "signed_call"
        if namespace in ("read", "sread")
        else "send_shielded_transaction"
    )
    with patch(
        f"seismic_web3.contract.shielded.{helper}",
        return_value=HexBytes("0x"),
    ) as mock:
        getattr(contract, namespace).setNumber(1, gas_payment=_PAYMENTS[-1])
        assert mock.call_args.kwargs["gas_payment"] == _PAYMENTS[-1]


@pytest.mark.parametrize("namespace", ["write", "swrite", "dwrite", "read", "sread"])
async def test_async_contract_forwarding(namespace):
    contract = AsyncShieldedContract(
        MagicMock(),
        _make_encryption(),
        ANVIL_PK,
        _ADDRESS,
        COUNTER_ABI,
    )
    helper = (
        "async_debug_send_shielded_transaction"
        if namespace == "dwrite"
        else "async_signed_call"
        if namespace in ("read", "sread")
        else "async_send_shielded_transaction"
    )
    with patch(
        f"seismic_web3.contract.shielded.{helper}",
        new_callable=AsyncMock,
        return_value=HexBytes("0x"),
    ) as mock:
        await getattr(contract, namespace).setNumber(1, gas_payment=_PAYMENTS[-1])
        assert mock.call_args.kwargs["gas_payment"] == _PAYMENTS[-1]


@pytest.mark.parametrize("namespace", ["write", "twrite"])
def test_transparent_paths_reject_before_rpc(namespace):
    w3 = MagicMock()
    contract = ShieldedContract(
        w3,
        _make_encryption(),
        ANVIL_PK,
        _ADDRESS,
        COUNTER_ABI,
    )
    with pytest.raises(ValueError, match="requires a Seismic"):
        getattr(contract, namespace).increment(gas_payment=GasPayment.native())
    w3.eth.send_transaction.assert_not_called()


@pytest.mark.parametrize(
    "method",
    ["send_shielded_transaction", "debug_send_shielded_transaction", "signed_call"],
)
def test_module_forwarding(method):
    namespace = SeismicNamespace(MagicMock(), _make_encryption(), ANVIL_PK)
    with patch(f"seismic_web3.module.{method}", return_value=HexBytes("0x")) as mock:
        getattr(namespace, method)(
            to="0x" + "33" * 20,
            data=HexBytes("0x"),
            gas_payment=_PAYMENTS[-1],
        )
        assert mock.call_args.kwargs["gas_payment"] == _PAYMENTS[-1]


@pytest.mark.parametrize(
    "method",
    ["send_shielded_transaction", "debug_send_shielded_transaction", "signed_call"],
)
async def test_async_module_forwarding(method):
    namespace = AsyncSeismicNamespace(MagicMock(), _make_encryption(), ANVIL_PK)
    with patch(
        f"seismic_web3.module.async_{method}",
        new_callable=AsyncMock,
        return_value=HexBytes("0x"),
    ) as mock:
        await getattr(namespace, method)(
            to="0x" + "33" * 20,
            data=HexBytes("0x"),
            gas_payment=_PAYMENTS[-1],
        )
        assert mock.call_args.kwargs["gas_payment"] == _PAYMENTS[-1]


def test_transparent_auto_preserves_standard_request():
    w3 = MagicMock()
    contract = ShieldedContract(w3, _make_encryption(), ANVIL_PK, _ADDRESS, COUNTER_ABI)
    contract.twrite.increment(gas=21_000, gasPrice=1, nonce=0, value=1)
    omitted = w3.eth.send_transaction.call_args.args[0]
    contract.twrite.increment(
        gas=21_000,
        gasPrice=1,
        nonce=0,
        value=1,
        gas_payment=GasPayment.auto(),
    )
    assert w3.eth.send_transaction.call_args.args[0] == omitted
    assert "gas_payment" not in omitted


async def test_async_transparent_auto_preserves_standard_request():
    w3 = MagicMock()
    w3.eth.send_transaction = AsyncMock(return_value=HexBytes(b"\x55" * 32))
    contract = AsyncShieldedContract(
        w3,
        _make_encryption(),
        ANVIL_PK,
        _ADDRESS,
        COUNTER_ABI,
    )
    await contract.twrite.increment(gas=21_000, gasPrice=1, nonce=0, value=1)
    omitted = w3.eth.send_transaction.call_args.args[0]
    await contract.twrite.increment(
        gas=21_000,
        gasPrice=1,
        nonce=0,
        value=1,
        gas_payment=GasPayment.auto(),
    )
    assert w3.eth.send_transaction.call_args.args[0] == omitted
    assert "gas_payment" not in omitted
