"""SDK funding is explicit and isolated from reth's production dev genesis."""

import hashlib
import json
from unittest.mock import patch

from tests.integration.conftest import (
    DEV_ADDRESS,
    RETH_TEST_GENESIS,
    _start_reth,
)

# Canonical JSON digest of dev.json at seismic-reth
# c4c048c153ce9043debed81a2efa6e6196eca19d.
# See clients/test-genesis/README.md for provenance and refresh instructions.
PRODUCTION_GENESIS_DIGEST = (
    "f16492f9a13ff416988ecea5ca0a03bbb78a4b07bdbdec0fddf20d612e9b34c9"
)


def test_only_sdk_account_funding_differs_from_production():
    genesis = json.loads(RETH_TEST_GENESIS.read_text())
    account = genesis["alloc"].pop(DEV_ADDRESS.lower())
    assert account == {"balance": "0xD3C21BCECCEDA1000000"}
    assert int(account["balance"], 16) == 1_000_000 * 10**18
    canonical = json.dumps(genesis, sort_keys=True, separators=(",", ":"))
    assert hashlib.sha256(canonical.encode()).hexdigest() == PRODUCTION_GENESIS_DIGEST


def test_python_reth_launcher_uses_shared_genesis(tmp_path, monkeypatch):
    monkeypatch.setenv("SRETH_BIN", "/test/seismic-reth")
    # Fixture resolution must not depend on invoking pytest from clients/py.
    monkeypatch.chdir(tmp_path)
    with (
        patch(
            "tests.integration.conftest.tempfile.mkdtemp",
            return_value=str(tmp_path),
        ),
        patch("tests.integration.conftest.subprocess.Popen") as popen,
    ):
        process, directory = _start_reth(18545)

    assert process is popen.return_value
    assert directory == str(tmp_path)
    command = popen.call_args.args[0]
    assert command[0] == "/test/seismic-reth"
    assert "--dev" in command
    assert command[command.index("--chain") + 1] == str(RETH_TEST_GENESIS)
    assert RETH_TEST_GENESIS.is_absolute()
    assert RETH_TEST_GENESIS.is_file()
