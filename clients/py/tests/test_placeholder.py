from importlib.metadata import version

import seismic_web3


def test_module_version_matches_distribution_metadata():
    assert seismic_web3.__version__ == version("seismic-web3")
