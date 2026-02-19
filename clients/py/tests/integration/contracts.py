"""Contract artifacts loader and deploy helper for integration tests.

ABIs and bytecodes live in JSON files under artifacts/.
When seismic-client moves into this repo, just update ARTIFACTS_DIR
to point at the shared location.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING, Any

from web3 import Web3

if TYPE_CHECKING:
    from eth_typing import ChecksumAddress

# ---------------------------------------------------------------------------
# Artifacts loading — change this path when contracts are centralized
# ---------------------------------------------------------------------------

ARTIFACTS_DIR: Path = Path(__file__).parent / "artifacts"


def _load_artifact(name: str) -> dict[str, Any]:
    with open(ARTIFACTS_DIR / name) as f:
        return json.load(f)


_seismic_counter: dict[str, Any] = _load_artifact("seismic_counter.json")
_transparent_counter: dict[str, Any] = _load_artifact("transparent_counter.json")

SEISMIC_COUNTER_ABI: list[dict[str, Any]] = _seismic_counter["abi"]
SEISMIC_COUNTER_BYTECODE: str = _seismic_counter["bytecode"]

TRANSPARENT_COUNTER_ABI: list[dict[str, Any]] = _transparent_counter["abi"]
TRANSPARENT_COUNTER_BYTECODE: str = _transparent_counter["bytecode"]

_test_token: dict[str, Any] = _load_artifact("test_token.json")

TEST_TOKEN_ABI: list[dict[str, Any]] = _test_token["abi"]
TEST_TOKEN_BYTECODE: str = _test_token["bytecode"]

_mock_src20_events: dict[str, Any] = _load_artifact("mock_src20_events.json")

MOCK_SRC20_EVENTS_ABI: list[dict[str, Any]] = _mock_src20_events["abi"]
MOCK_SRC20_EVENTS_BYTECODE: str = _mock_src20_events["bytecode"]


# ---------------------------------------------------------------------------
# Deploy helper
# ---------------------------------------------------------------------------


def deploy_contract(w3: Web3, bytecode: str, from_address: str) -> ChecksumAddress:
    """Deploy a contract and return its checksummed address."""
    tx_hash = w3.eth.send_transaction({"from": from_address, "data": bytecode})
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=30)
    assert receipt["status"] == 1, "Contract deployment failed"
    return Web3.to_checksum_address(receipt["contractAddress"])
