#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CONTRACTS_DIR="${CONTRACTS_DIR:-$WORKSPACE_ROOT/packages/erc4337-contracts}"
RPC_URL="${RPC_URL:-http://localhost:3050}"
BUNDLER_URL="${BUNDLER_URL:-http://localhost:4337}"
ENTRY_POINT="${ENTRY_POINT:-0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108}"
PRIVATE_KEY="${PRIVATE_KEY:-0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6}"
DEPLOYER_ADDRESS="${DEPLOYER_ADDRESS:-$(cast wallet address --private-key "$PRIVATE_KEY")}"
CHAIN_ID="${CHAIN_ID:-$(cast chain-id --rpc-url "$RPC_URL")}"
LOCAL_CONTRACTS_FILE="${LOCAL_CONTRACTS_FILE:-contracts.local.json}"
CONTRACTS_JSON_PATH="${CONTRACTS_JSON_PATH:-$WORKSPACE_ROOT/examples/demo-app/contracts.json}"
PUBLIC_CONTRACTS_JSON_PATH="${PUBLIC_CONTRACTS_JSON_PATH:-$WORKSPACE_ROOT/examples/demo-app/public/contracts.json}"
AUTH_SERVER_CONTRACTS_PATH="${AUTH_SERVER_CONTRACTS_PATH:-$WORKSPACE_ROOT/packages/auth-server/stores/contracts.json}"
AUTH_SERVER_API_CONTRACTS_PATH="${AUTH_SERVER_API_CONTRACTS_PATH:-$WORKSPACE_ROOT/packages/auth-server-api/src/contracts.json}"

echo "Deploying reusable SSO contract suite..."
echo ""
echo "📍 Deployer: $DEPLOYER_ADDRESS"
echo "🌐 RPC URL: $RPC_URL"
echo "⛓️  Chain ID: $CHAIN_ID"
echo ""

cd "$CONTRACTS_DIR"

# Build contracts first to ensure everything is compiled
echo "🔨 Building contracts..."
forge build

# Deploy the reusable factory/module suite only.
echo ""
echo "Deploying SSO contracts using pnpm deploy-contracts..."
DEPLOY_OUTPUT=$(RPC_URL="$RPC_URL" \
  PRIVATE_KEY="$PRIVATE_KEY" \
  pnpm deploy-contracts 2>&1)

echo "$DEPLOY_OUTPUT"

# Extract addresses from the deployment output
EOA_VALIDATOR=$(echo "$DEPLOY_OUTPUT" | grep "EOAKeyValidator:" | awk '{print $2}')
SESSION_VALIDATOR=$(echo "$DEPLOY_OUTPUT" | grep "SessionKeyValidator:" | awk '{print $2}')
WEBAUTHN_VALIDATOR=$(echo "$DEPLOY_OUTPUT" | grep "WebAuthnValidator:" | awk '{print $2}')
GUARDIAN_EXECUTOR=$(echo "$DEPLOY_OUTPUT" | grep "GuardianExecutor:" | awk '{print $2}')
ACCOUNT_IMPL=$(echo "$DEPLOY_OUTPUT" | grep "ModularSmartAccount implementation:" | awk '{print $3}')
BEACON=$(echo "$DEPLOY_OUTPUT" | grep "UpgradeableBeacon:" | awk '{print $2}')
FACTORY=$(echo "$DEPLOY_OUTPUT" | grep "MSAFactory:" | awk '{print $2}')

# Verify all addresses were extracted
if [ -z "$EOA_VALIDATOR" ] || [ -z "$SESSION_VALIDATOR" ] || [ -z "$WEBAUTHN_VALIDATOR" ] || \
  [ -z "$GUARDIAN_EXECUTOR" ] || [ -z "$ACCOUNT_IMPL" ] || [ -z "$BEACON" ] || [ -z "$FACTORY" ]; then
  echo "❌ Failed to extract all contract addresses from deployment output"
  echo "Please check the deployment logs above"
  exit 1
fi

echo ""
echo "Deployment complete"
echo "  EOAKeyValidator: $EOA_VALIDATOR"
echo "  SessionKeyValidator: $SESSION_VALIDATOR"
echo "  WebAuthnValidator: $WEBAUTHN_VALIDATOR"
echo "  GuardianExecutor: $GUARDIAN_EXECUTOR"
echo "  ModularSmartAccount impl: $ACCOUNT_IMPL"
echo "  UpgradeableBeacon: $BEACON"
echo "  SmartAccountFactory: $FACTORY"

# Create the local contracts manifest
echo ""
echo "Creating $LOCAL_CONTRACTS_FILE..."
cd "$WORKSPACE_ROOT/examples/demo-app"

cat > "$LOCAL_CONTRACTS_FILE" << EOF
{
  "rpcUrl": "$RPC_URL",
  "chainId": $CHAIN_ID,
  "deployer": "$DEPLOYER_ADDRESS",
  "eoaValidator": "$EOA_VALIDATOR",
  "sessionValidator": "$SESSION_VALIDATOR",
  "webauthnValidator": "$WEBAUTHN_VALIDATOR",
  "guardianExecutor": "$GUARDIAN_EXECUTOR",
  "accountImplementation": "$ACCOUNT_IMPL",
  "beacon": "$BEACON",
  "factory": "$FACTORY",
  "entryPoint": "$ENTRY_POINT",
  "bundlerUrl": "$BUNDLER_URL"
}
EOF

echo "Created $LOCAL_CONTRACTS_FILE"

# Also update contracts.json to point to the local deployment
cp "$LOCAL_CONTRACTS_FILE" "$CONTRACTS_JSON_PATH"
echo "Updated $CONTRACTS_JSON_PATH"

# Copy to public directory
cp "$LOCAL_CONTRACTS_FILE" "$PUBLIC_CONTRACTS_JSON_PATH"
echo "Copied to $PUBLIC_CONTRACTS_JSON_PATH"

# Copy to auth-server stores
cp "$LOCAL_CONTRACTS_FILE" "$AUTH_SERVER_CONTRACTS_PATH"
echo "Copied to $AUTH_SERVER_CONTRACTS_PATH"

# Copy to auth-server-api src
cp "$LOCAL_CONTRACTS_FILE" "$AUTH_SERVER_API_CONTRACTS_PATH"
echo "Copied to $AUTH_SERVER_API_CONTRACTS_PATH"

echo ""
echo "Contract addresses (local / Alto):"
cat "$LOCAL_CONTRACTS_FILE"
