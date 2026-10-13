from importlib.metadata import version

import seismic_web3


def test_import():
    assert seismic_web3.__version__ == version("seismic-web3")
