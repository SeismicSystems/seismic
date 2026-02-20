"""Session-scoped fixtures for integration tests.

Starts a real Seismic backend (sanvil or seismic-reth), shared across
the entire test session.

Environment variables:
    CHAIN              - "anvil" (default) or "reth"
    SEISMIC_PORT       - override the RPC port (default: pick a free port)
    SFOUNDRY_ROOT      - seismic-foundry repo root (target/debug/sanvil)
    SRETH_ROOT         - seismic-reth repo root (target/debug/seismic-reth)
    SEISMIC_WORKSPACE  - fallback: parent dir of seismic-foundry/ and seismic-reth/
"""

from __future__ import annotations

import os
import shutil
import signal
import socket
import subprocess
import tempfile
import time
from typing import TYPE_CHECKING

import pytest
import requests
from eth_account import Account
from web3 import Web3
from web3.middleware import SignAndSendRawMiddlewareBuilder

from seismic_web3 import (
    PrivateKey,
    create_async_wallet_client,
    create_public_client,
    create_wallet_client,
)

if TYPE_CHECKING:
    from collections.abc import Generator

    from eth_typing import ChecksumAddress
    from web3 import AsyncWeb3

# ---------------------------------------------------------------------------
# Pytest hook — print which backend is running at the top of the test output
# ---------------------------------------------------------------------------


def pytest_report_header() -> str:
    chain = os.environ.get("CHAIN")
    if chain is None:
        return "chain: CHAIN not set (defaulting to anvil)"
    return f"chain: {chain}"


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

# Dev account #0 (same key used by both sanvil and seismic-reth dev mode)
DEV_PRIVATE_KEY_HEX: str = (
    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)
DEV_PRIVATE_KEY: bytes = bytes.fromhex(DEV_PRIVATE_KEY_HEX)
DEV_ADDRESS: str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# Expected chain IDs per backend
CHAIN_IDS: dict[str, int] = {
    "anvil": 31_337,
    "reth": 5124,
}

# ---------------------------------------------------------------------------
# Binary resolution
# ---------------------------------------------------------------------------


def _resolve_binary(
    specific_env: str,
    workspace_subdir: str,
    relative_bin: str,
) -> str:
    """Resolve a binary path from environment variables.

    Resolution order:
        1. ``$<specific_env>/<relative_bin>``
        2. ``$SEISMIC_WORKSPACE/<workspace_subdir>/<relative_bin>``
        3. Raise with a clear message listing both options.
    """
    root = os.environ.get(specific_env)
    if root:
        return os.path.join(os.path.expanduser(root), relative_bin)

    workspace = os.environ.get("SEISMIC_WORKSPACE")
    if workspace:
        return os.path.join(
            os.path.expanduser(workspace), workspace_subdir, relative_bin
        )

    raise pytest.UsageError(
        f"Cannot locate {relative_bin!r}. "
        f"Set {specific_env} (repo root) or SEISMIC_WORKSPACE "
        f"(parent of all seismic repos).\n"
        f"Example:\n"
        f"  export {specific_env}=/path/to/{workspace_subdir}\n"
        f"  # or\n"
        f"  export SEISMIC_WORKSPACE=/path/to/seismic-workspace"
    )


def _get_sanvil_bin() -> str:
    return _resolve_binary(
        "SFOUNDRY_ROOT", "seismic-foundry", os.path.join("target", "debug", "sanvil")
    )


def _get_reth_bin() -> str:
    return _resolve_binary(
        "SRETH_ROOT", "seismic-reth", os.path.join("target", "debug", "seismic-reth")
    )


_StartResult = tuple[subprocess.Popen[bytes], str | None]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _wait_for_rpc(url: str, timeout: int = 15) -> None:
    """Poll until the node responds to eth_chainId."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            r = requests.post(
                url,
                json={
                    "jsonrpc": "2.0",
                    "method": "eth_chainId",
                    "params": [],
                    "id": 1,
                },
                timeout=2,
            )
            if r.status_code == 200 and "result" in r.json():
                return
        except (requests.ConnectionError, requests.Timeout):
            pass
        time.sleep(0.3)
    raise TimeoutError(f"Node at {url} did not start within {timeout}s")


def _start_anvil(port: int) -> _StartResult:
    cmd = [_get_sanvil_bin(), "--port", str(port), "--silent"]
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    return proc, None


def _start_reth(port: int) -> _StartResult:
    """Start seismic-reth in dev mode, matching seismic-viem's CI config."""
    tmpdir = tempfile.mkdtemp(prefix="seismic-reth-")
    cmd = [
        _get_reth_bin(),
        "node",
        # Dev mode
        "--dev",
        "--dev.block-max-transactions",
        "1",
        # HTTP RPC
        "--http",
        "--http.port",
        str(port),
        # Enclave
        "--enclave.mock-server",
        # Logging
        "--quiet",
        # Data directories
        "--datadir",
        tmpdir,
    ]
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    return proc, tmpdir


# ---------------------------------------------------------------------------
# Session-scoped fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(scope="session")
def chain() -> str:
    return os.environ.get("CHAIN", "anvil")


@pytest.fixture(scope="session")
def expected_chain_id(chain: str) -> int:
    return CHAIN_IDS[chain]


@pytest.fixture(scope="session")
def port() -> int:
    return int(os.environ.get("SEISMIC_PORT", _free_port()))


@pytest.fixture(scope="session")
def rpc_url(port: int) -> str:
    return f"http://127.0.0.1:{port}"


@pytest.fixture(scope="session")
def node_process(
    chain: str,
    port: int,
    rpc_url: str,
) -> Generator[subprocess.Popen[bytes]]:
    starters = {"anvil": _start_anvil, "reth": _start_reth}
    start_fn = starters.get(chain)
    if start_fn is None:
        pytest.fail(f"Unknown CHAIN={chain!r}. Use 'anvil' or 'reth'.")

    proc, tmpdir = start_fn(port)
    try:
        _wait_for_rpc(rpc_url, timeout=30)
        yield proc
    finally:
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
            proc.wait(timeout=10)
        except (ProcessLookupError, OSError):
            pass  # process already exited
        if tmpdir:
            shutil.rmtree(tmpdir, ignore_errors=True)


@pytest.fixture(scope="session")
def private_key() -> PrivateKey:
    return PrivateKey(DEV_PRIVATE_KEY)


@pytest.fixture(scope="session")
def account_address() -> ChecksumAddress:
    return Web3.to_checksum_address(DEV_ADDRESS)


@pytest.fixture(scope="session")
def w3(
    node_process: subprocess.Popen[bytes],
    rpc_url: str,
    private_key: PrivateKey,
    account_address: ChecksumAddress,
) -> Web3:
    w3 = create_wallet_client(rpc_url, private_key=private_key)
    # Add local signing so twrite works on reth (which has no unlocked keystore)
    acct = Account.from_key(DEV_PRIVATE_KEY_HEX)
    w3.middleware_onion.inject(
        SignAndSendRawMiddlewareBuilder.build(acct),
        layer=0,
    )
    w3.eth.default_account = account_address
    return w3


@pytest.fixture(scope="session")
def plain_w3(
    node_process: subprocess.Popen[bytes],
    rpc_url: str,
    account_address: ChecksumAddress,
) -> Web3:
    """Plain Web3 with local signing middleware for contract deployment."""
    acct = Account.from_key(DEV_PRIVATE_KEY_HEX)
    w3 = Web3(Web3.HTTPProvider(rpc_url))
    w3.middleware_onion.inject(
        SignAndSendRawMiddlewareBuilder.build(acct),
        layer=0,
    )
    w3.eth.default_account = account_address
    return w3


# ---------------------------------------------------------------------------
# Function-scoped async fixture
# ---------------------------------------------------------------------------


@pytest.fixture
async def async_w3(
    node_process: subprocess.Popen[bytes],
    rpc_url: str,
    private_key: PrivateKey,
) -> AsyncWeb3:
    return await create_async_wallet_client(rpc_url, private_key=private_key)


@pytest.fixture(scope="session")
def public_w3(
    node_process: subprocess.Popen[bytes],
    rpc_url: str,
) -> Web3:
    """Public (read-only) Web3 — no private key."""
    return create_public_client(rpc_url)
