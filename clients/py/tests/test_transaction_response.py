"""Existing sync/async web3.py lookups preserve typed, tagged response mappings."""

import json
from copy import deepcopy
from pathlib import Path
from unittest.mock import patch

import pytest
from hexbytes import HexBytes
from web3 import Web3
from web3.datastructures import AttributeDict
from web3.providers import AsyncBaseProvider, BaseProvider
from web3.types import TxData

from seismic_web3 import (
    GasPayment,
    GasPaymentResponse,
    PrivateKey,
    SeismicTransactionResponse,
    create_async_public_client,
    create_async_wallet_client,
    create_public_client,
    create_wallet_client,
    is_seismic_transaction,
)

FIXTURE = json.loads(
    (Path(__file__).parents[2] / "test-vectors/gas-payment.json").read_text(),
)
VECTORS = FIXTURE["vectors"]
BLOCK_HASH = "0x" + "ab" * 32
SENDER = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
KEY = PrivateKey(FIXTURE["privateKey"])


def raw_transaction(vector):
    return {
        **vector["tx"],
        **vector["signature"],
        "type": "0x4A",
        "blockHash": BLOCK_HASH,
        "blockNumber": "0x1",
        "transactionIndex": "0x0",
        "hash": vector["txHash"],
        "from": SENDER,
        "to": vector["tx"].get("to"),
    }


class FixtureProvider(BaseProvider):
    def __init__(self, transaction):
        super().__init__()
        self.transaction = transaction

    def make_request(self, method, params):
        if method in (
            "eth_getTransactionByHash",
            "eth_getTransactionByBlockHashAndIndex",
            "eth_getTransactionByBlockNumberAndIndex",
        ):
            result = deepcopy(self.transaction)
        elif method in ("eth_getBlockByHash", "eth_getBlockByNumber"):
            result = {
                "hash": BLOCK_HASH,
                "number": "0x1",
                "transactions": (
                    [deepcopy(self.transaction)]
                    if params[1]
                    else [self.transaction["hash"]]
                ),
            }
        elif method == "seismic_getTeePublicKey":
            result = "0x" + VECTORS[0]["tx"]["encryptionPubkey"]
        else:
            raise AssertionError(f"Unexpected response fixture RPC: {method}")
        return {"jsonrpc": "2.0", "id": 1, "result": result}


class AsyncFixtureProvider(AsyncBaseProvider):
    def __init__(self, provider):
        super().__init__()
        self.provider = provider

    async def make_request(self, method, params):
        return self.provider.make_request(method, params)


def sync_client(provider, wallet):
    with patch("seismic_web3.client.Web3.HTTPProvider", return_value=provider):
        if wallet:
            return create_wallet_client(
                "http://fixture.invalid", KEY, encryption_sk=KEY
            )
        return create_public_client("http://fixture.invalid")


async def async_client(provider, wallet):
    with patch(
        "seismic_web3.client.AsyncHTTPProvider",
        return_value=AsyncFixtureProvider(provider),
    ):
        if wallet:
            return await create_async_wallet_client(
                "http://fixture.invalid",
                KEY,
                encryption_sk=KEY,
            )
        return create_async_public_client("http://fixture.invalid")


def typed_payment(transaction: TxData | HexBytes) -> GasPaymentResponse | None:
    if is_seismic_transaction(transaction):
        response: SeismicTransactionResponse = transaction
        payment: GasPaymentResponse = response["gasPayment"]
        if payment["type"] == "token":
            token: str = payment["token"]
            assert isinstance(token, str)
        return payment
    return None


def assert_seismic_response(transaction, raw):
    assert is_seismic_transaction(transaction)
    # Static narrowing is checked separately by ty; no conversion is performed.
    payment = transaction["gasPayment"]
    assert isinstance(transaction, AttributeDict)
    assert isinstance(payment, AttributeDict)
    assert not isinstance(payment, GasPayment)
    assert dict(payment) == raw["gasPayment"]
    assert transaction["type"] == 74
    assert transaction["gas"] == int(raw["gas"], 16)
    assert transaction["input"] == HexBytes(raw["input"])
    for key in (
        "encryptionPubkey",
        "encryptionNonce",
        "messageVersion",
        "recentBlockHash",
        "expiresAtBlock",
        "signedRead",
    ):
        assert transaction[key] == raw[key]
    assert is_seismic_transaction(transaction)
    assert transaction["gasPayment"] is payment
    assert typed_payment(transaction) is payment


@pytest.mark.parametrize("wallet", [False, True], ids=["public", "wallet"])
@pytest.mark.parametrize("vector", VECTORS)
def test_sync_lookup_surfaces(vector, wallet):
    raw = raw_transaction(vector)
    w3 = sync_client(FixtureProvider(raw), wallet)
    transactions = [
        w3.eth.get_transaction(raw["hash"]),
        w3.eth.get_transaction_by_block(BLOCK_HASH, 0),
        w3.eth.get_transaction_by_block(1, 0),
        w3.eth.get_block(BLOCK_HASH, full_transactions=True)["transactions"][0],
        w3.eth.get_block(1, full_transactions=True)["transactions"][0],
    ]
    for transaction in transactions:
        assert_seismic_response(transaction, raw)
    hashes = w3.eth.get_block(BLOCK_HASH)["transactions"]
    assert hashes == [HexBytes(raw["hash"])]
    assert not is_seismic_transaction(hashes[0])


@pytest.mark.parametrize("wallet", [False, True], ids=["public", "wallet"])
@pytest.mark.parametrize("vector", VECTORS)
async def test_async_lookup_surfaces(vector, wallet):
    raw = raw_transaction(vector)
    w3 = await async_client(FixtureProvider(raw), wallet)
    transactions = [
        await w3.eth.get_transaction(raw["hash"]),
        await w3.eth.get_transaction_by_block(BLOCK_HASH, 0),
        await w3.eth.get_transaction_by_block(1, 0),
        (await w3.eth.get_block(BLOCK_HASH, full_transactions=True))["transactions"][0],
        (await w3.eth.get_block(1, full_transactions=True))["transactions"][0],
    ]
    for transaction in transactions:
        assert_seismic_response(transaction, raw)
    hashes = (await w3.eth.get_block(BLOCK_HASH))["transactions"]
    assert hashes == [HexBytes(raw["hash"])]
    assert not is_seismic_transaction(hashes[0])


@pytest.mark.parametrize("type_", ["0x0", "0x1", "0x2", "0x3", "0x4"])
async def test_ordinary_ethereum_responses_are_unchanged(type_):
    raw = {
        "type": type_,
        "hash": VECTORS[0]["txHash"],
        "blockHash": BLOCK_HASH,
        "blockNumber": "0x1",
        "transactionIndex": "0x0",
        "from": SENDER,
        "to": SENDER,
        "input": "0x",
        "value": "0x0",
        "gas": "0x5208",
        "gasPrice": "0x1",
        "nonce": "0x0",
        "v": "0x1",
        "r": "0x01",
        "s": "0x02",
    }
    baseline = Web3(FixtureProvider(raw)).eth.get_transaction(raw["hash"])
    sync = sync_client(FixtureProvider(raw), False)
    async_ = await async_client(FixtureProvider(raw), False)
    responses = [
        sync.eth.get_transaction(raw["hash"]),
        sync.eth.get_transaction_by_block(BLOCK_HASH, 0),
        sync.eth.get_block(BLOCK_HASH, full_transactions=True)["transactions"][0],
        await async_.eth.get_transaction(raw["hash"]),
        await async_.eth.get_transaction_by_block(BLOCK_HASH, 0),
        (await async_.eth.get_block(BLOCK_HASH, full_transactions=True))[
            "transactions"
        ][0],
    ]
    for transaction in responses:
        assert transaction == baseline
        assert not is_seismic_transaction(transaction)
        assert "gasPayment" not in transaction


@pytest.mark.parametrize(
    "payment",
    [
        None,
        {"type": "unknown"},
        {"type": "token"},
        {"type": "token", "token": "0x" + "00" * 20},
        {"type": "token", "token": "0x1234"},
        {"type": "native", "token": SENDER},
        {"type": "auto", "extra": True},
    ],
)
def test_malformed_or_missing_payment_is_not_defaulted(payment):
    raw = raw_transaction(VECTORS[0])
    if payment is None:
        del raw["gasPayment"]
    else:
        raw["gasPayment"] = payment
    transaction = Web3(FixtureProvider(raw)).eth.get_transaction(raw["hash"])
    before = dict(transaction)
    assert not is_seismic_transaction(transaction)
    assert dict(transaction) == before


def test_pending_create_and_zero_metadata_are_preserved():
    raw = raw_transaction(VECTORS[0])
    raw.update(
        blockHash=None,
        blockNumber=None,
        transactionIndex=None,
        to=None,
        encryptionNonce="0x0",
        messageVersion="0x0",
        expiresAtBlock="0x0",
    )
    transaction = sync_client(FixtureProvider(raw), False).eth.get_transaction(
        raw["hash"]
    )
    assert is_seismic_transaction(transaction)
    for key in ("blockHash", "blockNumber", "transactionIndex", "to"):
        assert transaction[key] is None
    assert transaction["encryptionNonce"] == "0x0"
    assert transaction["messageVersion"] == "0x0"
    assert transaction["expiresAtBlock"] == "0x0"


def test_public_response_typing_narrows_existing_methods():
    # This example is runtime exercised here and statically checked by ty.
    w3 = sync_client(FixtureProvider(raw_transaction(VECTORS[0])), False)
    transaction = w3.eth.get_transaction(VECTORS[0]["txHash"])
    if is_seismic_transaction(transaction):
        response: SeismicTransactionResponse = transaction
        payment: GasPaymentResponse = response["gasPayment"]
        if payment["type"] == "token":
            token: str = payment["token"]
            assert isinstance(token, str)
        else:
            assert payment["type"] in ("auto", "native")
