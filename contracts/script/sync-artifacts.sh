#!/bin/bash

set -e

CONTRACTS_FILE="script/genesis-contracts.txt"

# MeasurementRegistry's runtime code hash is pinned by the enclave crate, the
# seismic-tee CLI and deploy validation, so its artifact changes only in a
# coordinated re-pin across those repos. Drop it from this list for that bump.
FROZEN=(MeasurementRegistry)

if [ ! -f "$CONTRACTS_FILE" ]; then
    echo "❌ Error: $CONTRACTS_FILE not found"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "❌ Error: jq not found. Install: brew install jq (macOS) or apt install jq (Linux)"
    exit 1
fi

echo "Building contracts..."
mise run "${MISE_SFORGE_TASK:-sforge}" -- build

echo "Syncing genesis contracts..."
mkdir -p artifacts

synced=0
while IFS= read -r contract_name || [ -n "$contract_name" ]; do
    [[ -z "$contract_name" || "$contract_name" =~ ^#.*$ ]] && continue
    contract_name=$(echo "$contract_name" | xargs)

    if [[ " ${FROZEN[*]} " == *" ${contract_name} "* ]]; then
        echo "${contract_name}.json (frozen, not synced)"
        continue
    fi

    src="out/${contract_name}.sol/${contract_name}.json"
    dst="artifacts/${contract_name}.json"

    if [ -f "$src" ]; then
        # Only the fields consumers read. Source maps, ASTs and metadata change
        # with comments, paths and unrelated files, which would churn the check.
        jq '{abi, bytecode: {object: .bytecode.object}, deployedBytecode: {object: .deployedBytecode.object}, methodIdentifiers}' "$src" > "$dst"
        echo "${contract_name}.json"
        synced=$((synced + 1))
    else
        echo "  ${contract_name}.json not found in out/"
        exit 1
    fi
done < "$CONTRACTS_FILE"

echo "Synced $synced contracts to artifacts/"
