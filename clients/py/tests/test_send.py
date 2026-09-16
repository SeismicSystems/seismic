"""Tests for seismic_web3.transaction.send — address derivation, estimation."""

from unittest.mock import AsyncMock, MagicMock

import pytest
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from hexbytes import HexBytes
from web3.exceptions import ContractLogicError

from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)
from seismic_web3.client import get_encryption
from seismic_web3.crypto.aes import RESPONSE_FORMAT_VERSION
from seismic_web3.transaction.aead import encode_response_aad
from seismic_web3.transaction.metadata import async_build_metadata, build_metadata
from seismic_web3.transaction.send import (
    _address_from_key,
    _build_metadata_params,
    _raise_signed_rpc_error,
    async_signed_call,
    estimate_transparent_gas,
    signed_call,
)
from seismic_web3.transaction_types import (
    LegacyFields,
    SeismicElements,
    SeismicSecurityParams,
    TxSeismicMetadata,
)

# Anvil account #0
ANVIL_PK = PrivateKey(
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)
ANVIL_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

_NETWORK_PK = CompressedPublicKey(
    "0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
)
_CLIENT_SK = PrivateKey(
    "0xa30363336e1bb949185292a2a302de86e447d98f3a43d823c8c234d9e3e5ad77"
)


class TestAddressFromKey:
    def test_anvil_account_0(self):
        """Derive address from Anvil's well-known account #0 private key."""
        address = _address_from_key(ANVIL_PK)
        assert address.lower() == ANVIL_ADDRESS.lower()

    def test_returns_checksummed(self):
        """Address should be checksummed (mixed case)."""
        address = _address_from_key(ANVIL_PK)
        # Checksummed addresses have uppercase letters
        assert any(c.isupper() for c in address[2:])


class TestEstimateTransparentGas:
    """Transparent gas estimation must go through a Seismic (0x4a) tx.

    The node's raw-bytes ``eth_estimateGas`` path rejects plain signed
    transactions (a signed read must carry ``seismic_elements``), and
    unsigned estimation strips ``from`` and ``value``, which breaks
    payable and sender-dependent calls.  The only request form that
    preserves authenticated caller context is a provisional shielded
    transaction.
    """

    def test_submits_seismic_tx_to_estimate_gas(self):
        w3 = MagicMock()
        w3.eth.chain_id = 31337
        w3.eth.get_transaction_count.return_value = 7
        w3.eth.get_block.return_value = {
            "hash": b"\x11" * 32,
            "number": 100,
            "gasLimit": 30_000_000,
        }
        w3.eth.gas_price = 10**9
        w3.provider.make_request.return_value = {"result": "0x5208"}

        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        gas = estimate_transparent_gas(
            w3,
            to="0x5FbDB2315678afecb367f032d93F642f64180aa3",
            data="0xd09de08a",
            value=32 * 10**18,
            private_key=ANVIL_PK,
            encryption=encryption,
        )

        assert gas == 0x5208
        method, params = w3.provider.make_request.call_args[0]
        assert method == "eth_estimateGas"
        assert params[0].startswith("0x4a"), (
            "transparent estimation must submit a provisional Seismic "
            "(0x4a) transaction: the node rejects plain signed txs on the "
            "raw-bytes path and strips from/value from unsigned requests"
        )


class TestRaiseSignedRpcError:
    def _metadata(self):
        return TxSeismicMetadata(
            sender=ANVIL_ADDRESS,
            legacy_fields=LegacyFields(
                chain_id=31337,
                nonce=0,
                to="0xd3e8763675e4c425df46cc3b5c0f6cbdac396046",
                value=0,
            ),
            seismic_elements=SeismicElements(
                encryption_pubkey=_NETWORK_PK,
                encryption_nonce=EncryptionNonce("0x46a2b6020bba77fcb1e676a6"),
                message_version=0,
                recent_block_hash=Bytes32("0x" + "11" * 32),
                expires_at_block=100,
                signed_read=False,
            ),
        )

    def test_plaintext_revert_data_surfaces_as_contract_logic_error(self):
        """Revert data that is not a response envelope must not escape as ValueError."""
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        # Error(string) selector: valid revert data, not a signed-read envelope.
        plaintext_revert = "0x08c379a0" + "00" * 96
        response = {
            "error": {"message": "execution reverted", "data": plaintext_revert}
        }

        with pytest.raises(ContractLogicError):
            _raise_signed_rpc_error(response, encryption, self._metadata())

    def test_short_revert_data_surfaces_as_contract_logic_error(self):
        """Revert data shorter than the minimum envelope is also not a ValueError."""
        encryption = get_encryption(_NETWORK_PK, _CLIENT_SK)
        response = {"error": {"message": "execution reverted", "data": "0xdeadbeef"}}

        with pytest.raises(ContractLogicError):
            _raise_signed_rpc_error(response, encryption, self._metadata())


_BLOCK = {"number": 10, "hash": HexBytes(b"\x11" * 32)}


def _mock_w3(result: str) -> MagicMock:
    """A Web3 double whose eth_call returns ``result`` verbatim."""
    w3 = MagicMock()
    w3.eth.chain_id = 31337
    w3.eth.gas_price = 1_000_000_000
    w3.eth.get_transaction_count.return_value = 0
    w3.eth.get_block.return_value = _BLOCK
    w3.provider.make_request.return_value = {"result": result}
    return w3


def _mock_async_w3(result: str) -> MagicMock:
    """An AsyncWeb3 double whose eth_call returns ``result`` verbatim."""
    w3 = MagicMock()
    w3.eth = MagicMock()
    type(w3.eth).chain_id = _AwaitableProperty(31337)
    type(w3.eth).gas_price = _AwaitableProperty(1_000_000_000)
    w3.eth.get_transaction_count = AsyncMock(return_value=0)
    w3.eth.get_block = AsyncMock(return_value=_BLOCK)
    w3.provider.make_request = AsyncMock(return_value={"result": result})
    return w3


class _AwaitableProperty:
    """Descriptor returning an awaitable, mimicking AsyncWeb3's async properties."""

    def __init__(self, value):
        self._value = value

    def __get__(self, obj, objtype=None):
        async def _coro():
            return self._value

        return _coro()


def _empty_envelope(encryption, metadata) -> str:
    """The envelope a node emits for an empty signed-read result."""
    iv = b"\x07" * 12
    aad = encode_response_aad(metadata, RESPONSE_FORMAT_VERSION)
    tag = AESGCM(bytes(encryption.response_aes_key)).encrypt(iv, b"", aad)
    return HexBytes(bytes([RESPONSE_FORMAT_VERSION]) + iv + tag).to_0x_hex()


_PINNED = SeismicSecurityParams(
    encryption_nonce=EncryptionNonce(b"\x2a" * 12),
    recent_block_hash=Bytes32(b"\x11" * 32),
    expires_at_block=110,
)


class TestSignedCallRejectsBareResults:
    """The caller must not short-circuit a `0x` result ahead of decryption.

    Guards the layer the SEI-369 truncation bug lived at: a bare `0x` reaching
    the caller must fail, not decode as an authenticated empty result. Both the
    sync and async callers are separate code paths and need separate cover.
    """

    def _encryption(self):
        return get_encryption(_NETWORK_PK, _CLIENT_SK)

    def test_bare_0x_result_is_rejected(self):
        with pytest.raises(ValueError, match="shorter than the"):
            signed_call(
                _mock_w3("0x"),
                encryption=self._encryption(),
                private_key=ANVIL_PK,
                to=ANVIL_ADDRESS,
                data=HexBytes(b""),
            )

    def test_valid_empty_envelope_returns_empty_plaintext(self):
        encryption = self._encryption()
        w3 = _mock_w3("0x")
        params = _build_metadata_params(
            ANVIL_PK,
            encryption,
            ANVIL_ADDRESS,
            0,
            _PINNED,
            signed_read=True,
            eip712=False,
        )
        metadata = build_metadata(w3, params)
        w3.provider.make_request.return_value = {
            "result": _empty_envelope(encryption, metadata)
        }

        out = signed_call(
            w3,
            encryption=encryption,
            private_key=ANVIL_PK,
            to=ANVIL_ADDRESS,
            data=HexBytes(b""),
            security=_PINNED,
        )
        assert bytes(out) == b""

    async def test_async_bare_0x_result_is_rejected(self):
        with pytest.raises(ValueError, match="shorter than the"):
            await async_signed_call(
                _mock_async_w3("0x"),
                encryption=self._encryption(),
                private_key=ANVIL_PK,
                to=ANVIL_ADDRESS,
                data=HexBytes(b""),
            )

    async def test_async_valid_empty_envelope_returns_empty_plaintext(self):
        encryption = self._encryption()
        w3 = _mock_async_w3("0x")
        params = _build_metadata_params(
            ANVIL_PK,
            encryption,
            ANVIL_ADDRESS,
            0,
            _PINNED,
            signed_read=True,
            eip712=False,
        )
        metadata = await async_build_metadata(w3, params)
        w3.provider.make_request = AsyncMock(
            return_value={"result": _empty_envelope(encryption, metadata)}
        )

        out = await async_signed_call(
            w3,
            encryption=encryption,
            private_key=ANVIL_PK,
            to=ANVIL_ADDRESS,
            data=HexBytes(b""),
            security=_PINNED,
        )
        assert bytes(out) == b""
