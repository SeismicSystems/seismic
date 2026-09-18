from importlib.metadata import version

import seismic_web3


def test_import():
    assert seismic_web3.__version__ == "0.3.0"


def test_module_version_matches_installed_distribution_metadata() -> None:
    assert seismic_web3.__version__ == version("seismic-web3")
