"""Unit tests for block explorer URL helpers in seismic_web3."""

from hexbytes import HexBytes

from seismic_web3.chains import SANVIL, SEISMIC_TESTNET, ChainConfig
from seismic_web3.client import create_public_client
from seismic_web3.explorer import (
    address_explorer_url,
    block_explorer_url,
    get_explorer_url,
    token_explorer_url,
    tx_explorer_url,
)

TEST_EXPLORER_URL = "https://explorer.test"

CHAIN_WITH_EXPLORER = ChainConfig(
    chain_id=99999,
    rpc_url="http://unused.invalid",
    name="TestChain",
    explorer_url=TEST_EXPLORER_URL,
)

CHAIN_WITHOUT_EXPLORER = ChainConfig(
    chain_id=99998,
    rpc_url="http://unused.invalid",
    name="NoExplorerChain",
    explorer_url=None,
)

CHAIN_WITH_TRAILING_SLASH = ChainConfig(
    chain_id=99997,
    rpc_url="http://unused.invalid",
    name="TrailingSlashChain",
    explorer_url=f"{TEST_EXPLORER_URL}/",
)

SANVIL_TEST_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

SAMPLE_TX_HASH = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"

SAMPLE_BLOCK_NUMBER = 42


class TestGetExplorerUrl:
    def test_returns_null_for_none_chain(self):
        assert get_explorer_url(None) is None

    def test_returns_null_for_chain_without_explorer(self):
        assert get_explorer_url(CHAIN_WITHOUT_EXPLORER) is None

    def test_returns_base_url_without_options(self):
        assert get_explorer_url(CHAIN_WITH_EXPLORER) == TEST_EXPLORER_URL

    def test_strips_trailing_slash_on_base_url(self):
        assert get_explorer_url(CHAIN_WITH_TRAILING_SLASH) == TEST_EXPLORER_URL

    def test_builds_item_url(self):
        result = get_explorer_url(
            CHAIN_WITH_EXPLORER,
            item="tx",
            id=SAMPLE_TX_HASH,
        )
        assert result == f"{TEST_EXPLORER_URL}/tx/{SAMPLE_TX_HASH}"

    def test_builds_item_url_with_tab(self):
        result = get_explorer_url(
            CHAIN_WITH_EXPLORER,
            item="tx",
            id=SAMPLE_TX_HASH,
            tab="logs",
        )
        assert result == f"{TEST_EXPLORER_URL}/tx/{SAMPLE_TX_HASH}?tab=logs"


class TestTxExplorerUrl:
    def test_builds_correct_url(self):
        result = tx_explorer_url(
            CHAIN_WITH_EXPLORER,
            SAMPLE_TX_HASH,
        )
        assert result == f"{TEST_EXPLORER_URL}/tx/{SAMPLE_TX_HASH}"

    def test_accepts_hexbytes(self):
        result = tx_explorer_url(
            CHAIN_WITH_EXPLORER,
            HexBytes(SAMPLE_TX_HASH),
        )
        assert result == f"{TEST_EXPLORER_URL}/tx/{SAMPLE_TX_HASH}"

    def test_with_tab(self):
        result = tx_explorer_url(
            CHAIN_WITH_EXPLORER,
            SAMPLE_TX_HASH,
            tab="internal",
        )
        assert result == f"{TEST_EXPLORER_URL}/tx/{SAMPLE_TX_HASH}?tab=internal"

    def test_returns_null_without_chain(self):
        assert tx_explorer_url(None, SAMPLE_TX_HASH) is None

    def test_returns_null_without_tx_hash(self):
        assert tx_explorer_url(CHAIN_WITH_EXPLORER, None) is None

    def test_returns_null_without_explorer(self):
        assert tx_explorer_url(CHAIN_WITHOUT_EXPLORER, SAMPLE_TX_HASH) is None


class TestAddressExplorerUrl:
    def test_builds_correct_url(self):
        result = address_explorer_url(
            CHAIN_WITH_EXPLORER,
            SANVIL_TEST_ADDRESS,
        )
        assert result == f"{TEST_EXPLORER_URL}/address/{SANVIL_TEST_ADDRESS}"

    def test_with_tab(self):
        result = address_explorer_url(
            CHAIN_WITH_EXPLORER,
            SANVIL_TEST_ADDRESS,
            tab="tokens",
        )
        assert result == f"{TEST_EXPLORER_URL}/address/{SANVIL_TEST_ADDRESS}?tab=tokens"

    def test_returns_null_without_chain(self):
        assert address_explorer_url(None, SANVIL_TEST_ADDRESS) is None

    def test_returns_null_without_address(self):
        assert address_explorer_url(CHAIN_WITH_EXPLORER, None) is None

    def test_returns_null_without_explorer(self):
        assert address_explorer_url(CHAIN_WITHOUT_EXPLORER, SANVIL_TEST_ADDRESS) is None


class TestBlockExplorerUrl:
    def test_builds_correct_url(self):
        result = block_explorer_url(
            CHAIN_WITH_EXPLORER,
            SAMPLE_BLOCK_NUMBER,
        )
        assert result == f"{TEST_EXPLORER_URL}/block/{SAMPLE_BLOCK_NUMBER}"

    def test_with_tab(self):
        result = block_explorer_url(
            CHAIN_WITH_EXPLORER,
            SAMPLE_BLOCK_NUMBER,
            tab="txs",
        )
        assert result == f"{TEST_EXPLORER_URL}/block/{SAMPLE_BLOCK_NUMBER}?tab=txs"

    def test_returns_null_without_chain(self):
        assert block_explorer_url(None, SAMPLE_BLOCK_NUMBER) is None

    def test_returns_null_without_block_number(self):
        assert block_explorer_url(CHAIN_WITH_EXPLORER, None) is None

    def test_returns_null_without_explorer(self):
        assert block_explorer_url(CHAIN_WITHOUT_EXPLORER, SAMPLE_BLOCK_NUMBER) is None


class TestTokenExplorerUrl:
    def test_builds_correct_url(self):
        result = token_explorer_url(
            CHAIN_WITH_EXPLORER,
            SANVIL_TEST_ADDRESS,
        )
        assert result == f"{TEST_EXPLORER_URL}/token/{SANVIL_TEST_ADDRESS}"

    def test_with_tab(self):
        result = token_explorer_url(
            CHAIN_WITH_EXPLORER,
            SANVIL_TEST_ADDRESS,
            tab="holders",
        )
        assert result == f"{TEST_EXPLORER_URL}/token/{SANVIL_TEST_ADDRESS}?tab=holders"

    def test_returns_null_without_chain(self):
        assert token_explorer_url(None, SANVIL_TEST_ADDRESS) is None

    def test_returns_null_without_address(self):
        assert token_explorer_url(CHAIN_WITH_EXPLORER, None) is None

    def test_returns_null_without_explorer(self):
        assert token_explorer_url(CHAIN_WITHOUT_EXPLORER, SANVIL_TEST_ADDRESS) is None


class TestPredefinedChains:
    def test_seismic_testnet_has_explorer(self):
        assert SEISMIC_TESTNET.explorer_url == "https://seismic-testnet.socialscan.io"
        result = tx_explorer_url(SEISMIC_TESTNET, SAMPLE_TX_HASH)
        expected = f"https://seismic-testnet.socialscan.io/tx/{SAMPLE_TX_HASH}"
        assert result == expected

    def test_sanvil_has_no_explorer(self):
        assert SANVIL.explorer_url is None
        assert tx_explorer_url(SANVIL, SAMPLE_TX_HASH) is None


class TestChainConfigExplorerMethods:
    def test_chain_methods_match_standalone(self):
        cfg = CHAIN_WITH_EXPLORER
        assert cfg.get_explorer_url() == TEST_EXPLORER_URL
        assert (
            cfg.tx_explorer_url(SAMPLE_TX_HASH)
            == f"{TEST_EXPLORER_URL}/tx/{SAMPLE_TX_HASH}"
        )
        assert (
            cfg.address_explorer_url(SANVIL_TEST_ADDRESS)
            == f"{TEST_EXPLORER_URL}/address/{SANVIL_TEST_ADDRESS}"
        )
        assert (
            cfg.block_explorer_url(SAMPLE_BLOCK_NUMBER)
            == f"{TEST_EXPLORER_URL}/block/{SAMPLE_BLOCK_NUMBER}"
        )
        assert (
            cfg.token_explorer_url(SANVIL_TEST_ADDRESS)
            == f"{TEST_EXPLORER_URL}/token/{SANVIL_TEST_ADDRESS}"
        )


class TestNamespaceExplorerIntegration:
    def test_public_client_with_chain(self):
        w3 = CHAIN_WITH_EXPLORER.public_client()
        assert w3.seismic.explorer_url() == TEST_EXPLORER_URL
        assert (
            w3.seismic.tx_explorer_url(SAMPLE_TX_HASH)
            == f"{TEST_EXPLORER_URL}/tx/{SAMPLE_TX_HASH}"
        )
        assert (
            w3.seismic.address_explorer_url(SANVIL_TEST_ADDRESS, tab="logs")
            == f"{TEST_EXPLORER_URL}/address/{SANVIL_TEST_ADDRESS}?tab=logs"
        )
        assert (
            w3.seismic.block_explorer_url(SAMPLE_BLOCK_NUMBER)
            == f"{TEST_EXPLORER_URL}/block/{SAMPLE_BLOCK_NUMBER}"
        )
        assert (
            w3.seismic.token_explorer_url(SANVIL_TEST_ADDRESS, tab="contract")
            == f"{TEST_EXPLORER_URL}/token/{SANVIL_TEST_ADDRESS}?tab=contract"
        )

    def test_public_client_without_chain(self):
        w3 = create_public_client("http://localhost:8545")
        assert w3.seismic.explorer_url() is None
        assert w3.seismic.tx_explorer_url(SAMPLE_TX_HASH) is None

    def test_public_client_sanvil_has_no_explorer(self):
        w3 = SANVIL.public_client()
        assert w3.seismic.explorer_url() is None
        assert w3.seismic.tx_explorer_url(SAMPLE_TX_HASH) is None
