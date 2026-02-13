#!/bin/bash

set -e

CONTRACTS_FILE="script/genesis-contracts.txt"

if [ ! -f "$CONTRACTS_FILE" ]; then
    echo "❌ Error: $CONTRACTS_FILE not found"
    exit 1
fi

# Check if jq is installed for JSON formatting
if ! command -v jq &> /dev/null; then
    echo "Warning: jq not found. JSON will not be formatted."
    echo "Install: brew install jq (macOS) or apt install jq (Linux)"
    USE_JQ=false
else
    USE_JQ=true
fi

echo "Building contracts..."
sforge build --libraries lib/AesLib.sol:AesLib:0x1000000000000000000000000000000000000003

echo "Syncing genesis contracts..."
mkdir -p artifacts

synced=0
while IFS= read -r contract_name || [ -n "$contract_name" ]; do
    [[ -z "$contract_name" || "$contract_name" =~ ^#.*$ ]] && continue
    contract_name=$(echo "$contract_name" | xargs)
    
    src="out/${contract_name}.sol/${contract_name}.json"
    dst="artifacts/${contract_name}.json"
    
    if [ -f "$src" ]; then
        if [ "$USE_JQ" = true ]; then
            # Format JSON with jq
            jq '.' "$src" > "$dst"
            echo "${contract_name}.json (formatted)"
        else
            # Just copy without formatting
            cp "$src" "$dst"
            echo "${contract_name}.json"
        fi
        synced=$((synced + 1))
    else
        echo "  ${contract_name}.json not found in out/"
        exit 1
    fi
done < "$CONTRACTS_FILE"

echo "Synced $synced contracts to artifacts/"
