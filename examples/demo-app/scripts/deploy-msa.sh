#!/bin/bash
set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Navigate to the workspace root (3 levels up from scripts/)
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Configuration
DEPLOYER_ADDRESS="0xa0Ee7A142d267C1f36714E4a8F75612F20a79720" # Local zksync-os rich wallet #9
RPC_URL="http://localhost:3050"
CHAIN_ID=6565
CONTRACTS_DIR="$WORKSPACE_ROOT/packages/erc4337-contracts"

echo "Deploying MSA Factory and modules to local zksync-os..."
echo ""
echo "📍 Deployer: $DEPLOYER_ADDRESS"
echo "🌐 RPC URL: $RPC_URL"
echo ""

cd "$CONTRACTS_DIR"

# Build contracts first to ensure everything is compiled
echo "🔨 Building contracts..."
forge build

# Use pnpm deploy-test to deploy all contracts (except paymaster)
echo ""
echo "Deploying MSA contracts using pnpm deploy-test..."
DEPLOY_OUTPUT=$(pnpm deploy-test 2>&1)

echo "$DEPLOY_OUTPUT"

# Extract addresses from the deployment output
EOA_VALIDATOR=$(echo "$DEPLOY_OUTPUT" | grep "EOAKeyValidator:" | awk '{print $2}')
SESSION_VALIDATOR=$(echo "$DEPLOY_OUTPUT" | grep "SessionKeyValidator:" | awk '{print $2}')
WEBAUTHN_VALIDATOR=$(echo "$DEPLOY_OUTPUT" | grep "WebAuthnValidator:" | awk '{print $2}')
GUARDIAN_EXECUTOR=$(echo "$DEPLOY_OUTPUT" | grep "GuardianExecutor:" | awk '{print $2}')
ACCOUNT_IMPL=$(echo "$DEPLOY_OUTPUT" | grep "ModularSmartAccount implementation:" | awk '{print $3}')
BEACON=$(echo "$DEPLOY_OUTPUT" | grep "UpgradeableBeacon:" | awk '{print $2}')
FACTORY=$(echo "$DEPLOY_OUTPUT" | grep "MSAFactory:" | awk '{print $2}')

# Deploy MockPaymaster directly from erc4337-contracts (simpler, no dependencies)
echo ""
echo "📦 Deploying MockPaymaster..."
cd "$CONTRACTS_DIR"
RICH_WALLET_KEY="0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6"
PAYMASTER_OUTPUT=$(forge create test/mocks/MockPaymaster.sol:MockPaymaster --rpc-url "$RPC_URL" --private-key "$RICH_WALLET_KEY" --broadcast 2>&1)
echo "$PAYMASTER_OUTPUT"
PAYMASTER=$(echo "$PAYMASTER_OUTPUT" | grep "Deployed to:" | awk '{print $3}')

echo "MockPaymaster deployed to: $PAYMASTER"

# Fund the paymaster with ETH from the local zksync-os rich wallet
echo ""
echo "Funding paymaster with 10 ETH..."
cast send "$PAYMASTER" --value 10ether --private-key "$RICH_WALLET_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Fund transfer initiated"

# Deposit the paymaster's ETH into the EntryPoint
echo "Depositing 10 ETH into EntryPoint for paymaster..."
cast send "$PAYMASTER" "deposit()" --value 10ether --private-key "$RICH_WALLET_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Deposit initiated"

# Add stake to the paymaster (required for ERC-4337)
echo "Adding stake to paymaster (1 day unlock delay)..."
cast send "$PAYMASTER" "addStake(uint32)" 86400 --value 1ether --private-key "$RICH_WALLET_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Stake added"

# Verify all addresses were extracted
if [ -z "$EOA_VALIDATOR" ] || [ -z "$SESSION_VALIDATOR" ] || [ -z "$WEBAUTHN_VALIDATOR" ] || \
  [ -z "$GUARDIAN_EXECUTOR" ] || [ -z "$ACCOUNT_IMPL" ] || [ -z "$BEACON" ] || [ -z "$FACTORY" ] || [ -z "$PAYMASTER" ]; then
  echo "❌ Failed to extract all contract addresses from deployment output"
  echo "Please check the deployment logs above"
  exit 1
fi

echo ""
echo "Deployment complete"
echo "  MockPaymaster: $PAYMASTER"
echo "  EOAKeyValidator: $EOA_VALIDATOR"
echo "  SessionKeyValidator: $SESSION_VALIDATOR"
echo "  WebAuthnValidator: $WEBAUTHN_VALIDATOR"
echo "  GuardianExecutor: $GUARDIAN_EXECUTOR"
echo "  ModularSmartAccount impl: $ACCOUNT_IMPL"
echo "  UpgradeableBeacon: $BEACON"
echo "  MSAFactory: $FACTORY"

# Create the local contracts manifest
echo ""
LOCAL_CONTRACTS_FILE="contracts.local.json"

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
  "testPaymaster": "$PAYMASTER",
  "mockPaymaster": "$PAYMASTER",
  "entryPoint": "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108",
  "bundlerUrl": "http://localhost:4337"
}
EOF

echo "Created $LOCAL_CONTRACTS_FILE"

# Also update contracts.json to point to the local deployment
cp "$LOCAL_CONTRACTS_FILE" contracts.json
echo "Updated contracts.json to use local addresses"

# Copy to public directory
cp "$LOCAL_CONTRACTS_FILE" public/contracts.json
echo "Copied to public/contracts.json"

# Copy to auth-server stores
cp "$LOCAL_CONTRACTS_FILE" "$WORKSPACE_ROOT/packages/auth-server/stores/contracts.json"
echo "Copied to packages/auth-server/stores/contracts.json"

# Copy to auth-server-api src
cp "$LOCAL_CONTRACTS_FILE" "$WORKSPACE_ROOT/packages/auth-server-api/src/contracts.json"
echo "Copied to packages/auth-server-api/src/contracts.json"

echo ""
echo "Contract addresses (local / Alto):"
cat "$LOCAL_CONTRACTS_FILE"
