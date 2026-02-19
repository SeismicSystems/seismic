"""Tests for seismic_web3.chains — chain definitions and constants."""

from seismic_web3.chains import (
    SANVIL,
    SANVIL_CHAIN_ID,
    SEISMIC_TESTNET,
    SEISMIC_TESTNET_CHAIN_ID,
    SEISMIC_TX_TYPE,
    TYPED_DATA_MESSAGE_VERSION,
    ChainConfig,
    make_seismic_testnet,
)


class TestConstants:
    def test_tx_type(self):
        assert SEISMIC_TX_TYPE == 0x4A
        assert SEISMIC_TX_TYPE == 74

    def test_typed_data_version(self):
        assert TYPED_DATA_MESSAGE_VERSION == 2

    def test_chain_ids(self):
        assert SEISMIC_TESTNET_CHAIN_ID == 5124
        assert SANVIL_CHAIN_ID == 31_337


class TestChainConfig:
    def test_frozen(self):
        cfg = ChainConfig(chain_id=1, rpc_url="http://localhost")
        try:
            cfg.chain_id = 2  # type: ignore[misc]
            raise AssertionError("expected FrozenInstanceError")
        except AttributeError:
            pass

    def test_defaults(self):
        cfg = ChainConfig(chain_id=1, rpc_url="http://localhost")
        assert cfg.ws_url is None
        assert cfg.name == ""


class TestSeismicTestnet:
    def test_chain_id(self):
        assert SEISMIC_TESTNET.chain_id == 5124

    def test_rpc_url(self):
        assert SEISMIC_TESTNET.rpc_url == "https://gcp-1.seismictest.net/rpc"

    def test_ws_url(self):
        assert SEISMIC_TESTNET.ws_url == "wss://gcp-1.seismictest.net/ws"

    def test_name(self):
        assert "Testnet" in SEISMIC_TESTNET.name


class TestMakeSeismicTestnet:
    def test_instance_2(self):
        cfg = make_seismic_testnet(2)
        assert cfg.chain_id == 5124
        assert "gcp-2" in cfg.rpc_url
        assert "gcp-2" in (cfg.ws_url or "")

    def test_default_is_1(self):
        cfg = make_seismic_testnet()
        assert cfg.rpc_url == SEISMIC_TESTNET.rpc_url


class TestSanvil:
    def test_chain_id(self):
        assert SANVIL.chain_id == 31_337

    def test_rpc_url(self):
        assert SANVIL.rpc_url == "http://127.0.0.1:8545"

    def test_ws_url(self):
        assert SANVIL.ws_url == "ws://127.0.0.1:8545"
