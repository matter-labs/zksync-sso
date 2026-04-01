#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CONTRACTS_DIR="${CONTRACTS_DIR:-$WORKSPACE_ROOT/packages/erc4337-contracts}"
L1_RPC_URL="${L1_RPC_URL:-http://localhost:5010}"
RPC_URL="${RPC_URL:-http://localhost:3050}"
PRIVATE_KEY="${PRIVATE_KEY:-0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6}"
MIN_L2_BALANCE="${MIN_L2_BALANCE:-15}"
BRIDGE_AMOUNT="${BRIDGE_AMOUNT:-25}"
ENABLE_L1_TO_L2_BRIDGE="${ENABLE_L1_TO_L2_BRIDGE:-true}"
BRIDGE_TEST_WALLETS="${BRIDGE_TEST_WALLETS:-true}"
LOCAL_TEST_WALLET_PRIVATE_KEYS="${LOCAL_TEST_WALLET_PRIVATE_KEYS:-0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a}"
PAYMASTER_FUNDING_WEI="${PAYMASTER_FUNDING_WEI:-10000000000000000000}"
PAYMASTER_STAKE_WEI="${PAYMASTER_STAKE_WEI:-1000000000000000000}"
PAYMASTER_UNLOCK_DELAY="${PAYMASTER_UNLOCK_DELAY:-86400}"
LOCAL_CONTRACTS_FILE="${LOCAL_CONTRACTS_FILE:-$WORKSPACE_ROOT/examples/demo-app/contracts.local.json}"
CONTRACTS_JSON_PATH="${CONTRACTS_JSON_PATH:-$WORKSPACE_ROOT/examples/demo-app/contracts.json}"
PUBLIC_CONTRACTS_JSON_PATH="${PUBLIC_CONTRACTS_JSON_PATH:-$WORKSPACE_ROOT/examples/demo-app/public/contracts.json}"
AUTH_SERVER_CONTRACTS_PATH="${AUTH_SERVER_CONTRACTS_PATH:-$WORKSPACE_ROOT/packages/auth-server/stores/contracts.json}"
AUTH_SERVER_API_CONTRACTS_PATH="${AUTH_SERVER_API_CONTRACTS_PATH:-$WORKSPACE_ROOT/packages/auth-server-api/src/contracts.json}"

bridge_wallet_if_needed() {
  local wallet_private_key="$1"
  local wallet_address
  wallet_address="$(cast wallet address --private-key "$wallet_private_key")"

  echo "Checking L2 balance for $wallet_address..."
  (
    cd "$CONTRACTS_DIR"
    PRIVATE_KEY="$wallet_private_key" \
      L1_RPC_URL="$L1_RPC_URL" \
      L2_RPC_URL="$RPC_URL" \
      MIN_L2_BALANCE="$MIN_L2_BALANCE" \
      BRIDGE_AMOUNT="$BRIDGE_AMOUNT" \
      pnpm bridge-funds
  )
}

update_manifest_with_paymaster() {
  local manifest_path="$1"
  local paymaster="$2"

  /usr/local/bin/node -e '
const fs = require("fs");
const manifestPath = process.argv[1];
const paymaster = process.argv[2];
const json = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
json.testPaymaster = paymaster;
json.mockPaymaster = paymaster;
fs.writeFileSync(manifestPath, `${JSON.stringify(json, null, 2)}\n`);
' "$manifest_path" "$paymaster"
}

if [ "$ENABLE_L1_TO_L2_BRIDGE" = "true" ]; then
  echo "Bridging deployer funds from L1 to L2 if needed..."
  bridge_wallet_if_needed "$PRIVATE_KEY"

  if [ "$BRIDGE_TEST_WALLETS" = "true" ]; then
    echo "Bridging local ERC-4337 test wallets from L1 to L2 if needed..."
    IFS=',' read -r -a test_wallet_keys <<< "$LOCAL_TEST_WALLET_PRIVATE_KEYS"
    for wallet_private_key in "${test_wallet_keys[@]}"; do
      bridge_wallet_if_needed "$wallet_private_key"
    done
  fi
else
  echo "Skipping L1 to L2 bridge step because ENABLE_L1_TO_L2_BRIDGE=$ENABLE_L1_TO_L2_BRIDGE"
fi

"$SCRIPT_DIR/deploy-sso-contracts.sh"

echo ""
echo "📦 Deploying MockPaymaster for local development..."
cd "$CONTRACTS_DIR"
PAYMASTER_OUTPUT=$(forge create test/mocks/MockPaymaster.sol:MockPaymaster --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" --broadcast --legacy 2>&1)
echo "$PAYMASTER_OUTPUT"
PAYMASTER=$(echo "$PAYMASTER_OUTPUT" | grep "Deployed to:" | awk '{print $3}')

if [ -z "$PAYMASTER" ]; then
  echo "❌ Failed to extract MockPaymaster address from deployment output"
  exit 1
fi

echo "Funding paymaster with $PAYMASTER_FUNDING_WEI wei..."
cast send "$PAYMASTER" --legacy --value "$PAYMASTER_FUNDING_WEI" --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Fund transfer initiated"

echo "Depositing $PAYMASTER_FUNDING_WEI wei into EntryPoint for paymaster..."
cast send "$PAYMASTER" --legacy "deposit()" --value "$PAYMASTER_FUNDING_WEI" --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Deposit initiated"

echo "Adding stake to paymaster (unlock delay $PAYMASTER_UNLOCK_DELAY)..."
cast send "$PAYMASTER" --legacy "addStake(uint32)" "$PAYMASTER_UNLOCK_DELAY" --value "$PAYMASTER_STAKE_WEI" --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Stake added"

echo ""
echo "Updating local manifests with MockPaymaster: $PAYMASTER"
update_manifest_with_paymaster "$LOCAL_CONTRACTS_FILE" "$PAYMASTER"
update_manifest_with_paymaster "$CONTRACTS_JSON_PATH" "$PAYMASTER"
update_manifest_with_paymaster "$PUBLIC_CONTRACTS_JSON_PATH" "$PAYMASTER"
update_manifest_with_paymaster "$AUTH_SERVER_CONTRACTS_PATH" "$PAYMASTER"
update_manifest_with_paymaster "$AUTH_SERVER_API_CONTRACTS_PATH" "$PAYMASTER"

echo "Local deployment complete"
cat "$LOCAL_CONTRACTS_FILE"
