#!/usr/bin/env bash
set -euo pipefail

# Deploy the ERC1271Caller helper for the demo app and emit forge-output-erc1271.json

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
DEMO_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
# (Optional) Workspace root; uncomment if needed later
# WORKSPACE_ROOT=$(cd "$SCRIPT_DIR/../../.." && pwd)

# Defaults for local Anvil
RPC_URL=${RPC_URL:-http://localhost:8545}
PRIVATE_KEY=${PRIVATE_KEY:-0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6} # anvil acct #9

CONTRACT_PATH="$DEMO_DIR/smart-contracts/ERC1271Caller.sol"
CONTRACT_NAME="ERC1271Caller"

echo "🚀 Deploying $CONTRACT_NAME to $RPC_URL"

if [[ ! -f "$CONTRACT_PATH" ]]; then
  echo "❌ Contract not found at $CONTRACT_PATH" >&2
  exit 1
fi

# Change to demo-app dir so forge can find remappings.txt and node_modules
cd "$DEMO_DIR"

# Build contract first to ensure it's compiled
forge build "smart-contracts/ERC1271Caller.sol" >/dev/null 2>&1

# Deploy and capture the address from forge output (EVM)
DEPLOY_OUTPUT=$(forge create "smart-contracts/ERC1271Caller.sol:$CONTRACT_NAME" \
  --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  --broadcast \
  2>&1)

# Extract deployed address from output (format: "Deployed to: 0x...")
DEPLOYED_TO=$(echo "$DEPLOY_OUTPUT" | grep -i "Deployed to:" | awk '{print $3}')

if [[ -z "${DEPLOYED_TO:-}" || ! "$DEPLOYED_TO" =~ ^0x[0-9a-fA-F]{40}$ ]]; then
  echo "❌ Failed to parse deployed address from forge output" >&2
  echo "$DEPLOY_OUTPUT"
  exit 1
fi

echo "✅ Deployed at: $DEPLOYED_TO"

# Write the output JSON used by the demo component
OUTPUT_FILE="$DEMO_DIR/forge-output-erc1271.json"
echo "{\"deployedTo\":\"$DEPLOYED_TO\"}" > "$OUTPUT_FILE"
echo "💾 Wrote $OUTPUT_FILE"

exit 0
