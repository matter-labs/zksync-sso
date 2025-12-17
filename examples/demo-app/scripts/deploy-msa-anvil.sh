#!/bin/bash
set -ex

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Navigate to the workspace root (3 levels up from scripts/)
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Configuration
DEPLOYER_ADDRESS="0xa0Ee7A142d267C1f36714E4a8F75612F20a79720"  # Anvil account #9
RPC_URL="http://localhost:8545"
CHAIN_ID=1337
CONTRACTS_DIR="$WORKSPACE_ROOT/packages/erc4337-contracts"

echo "🚀 Deploying MSA Factory and modules to Anvil (standard EVM)..."
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
echo "📦 Deploying MSA contracts using pnpm deploy-test..."
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

# Deploy paymaster separately (with dependencies from erc4337-contracts)
echo ""
echo "📦 Deploying TestPaymaster..."
PAYMASTER_OUTPUT=$(cd "$WORKSPACE_ROOT/examples/demo-app/smart-contracts" && forge script DeployPaymaster.s.sol:DeployPaymaster --rpc-url "$RPC_URL" --broadcast --private-key 0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6 2>&1)

echo "$PAYMASTER_OUTPUT"

PAYMASTER=$(echo "$PAYMASTER_OUTPUT" | grep "TestPaymaster:" | awk '{print $2}')

# Fund the paymaster with ETH from Anvil account #0 (has plenty of ETH)
echo ""
echo "💰 Funding paymaster with 10 ETH..."
ANVIL_ACCOUNT_0_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
cast send "$PAYMASTER" --value 10ether --private-key "$ANVIL_ACCOUNT_0_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Fund transfer initiated"

# Deposit the paymaster's ETH into the EntryPoint
echo "💳 Depositing 10 ETH into EntryPoint for paymaster..."
cast send "$PAYMASTER" "deposit()" --value 10ether --private-key "$ANVIL_ACCOUNT_0_KEY" --rpc-url "$RPC_URL" 2>&1 || echo "Deposit initiated"

# Verify all addresses were extracted
if [ -z "$EOA_VALIDATOR" ] || [ -z "$SESSION_VALIDATOR" ] || [ -z "$WEBAUTHN_VALIDATOR" ] || \
  [ -z "$GUARDIAN_EXECUTOR" ] || [ -z "$ACCOUNT_IMPL" ] || [ -z "$BEACON" ] || [ -z "$FACTORY" ] || [ -z "$PAYMASTER" ]; then
  echo "❌ Failed to extract all contract addresses from deployment output"
  echo "Please check the deployment logs above"
  exit 1
fi

echo ""
echo "✅ Deployment complete!"
echo "  EOAKeyValidator: $EOA_VALIDATOR"
echo "  SessionKeyValidator: $SESSION_VALIDATOR"
echo "  WebAuthnValidator: $WEBAUTHN_VALIDATOR"
echo "  GuardianExecutor: $GUARDIAN_EXECUTOR"
echo "  ModularSmartAccount impl: $ACCOUNT_IMPL"
echo "  UpgradeableBeacon: $BEACON"
echo "  MSAFactory: $FACTORY"
echo "  TestPaymaster: $PAYMASTER"

# Create contracts-anvil.json
echo ""
echo "💾 Creating contracts-anvil.json..."
cd "$WORKSPACE_ROOT/examples/demo-app"

cat > contracts-anvil.json << EOF
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
  "entryPoint": "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108",
  "bundlerUrl": "http://localhost:4337"
}
EOF

echo "✅ Created contracts-anvil.json"

# Also update contracts.json to point to Anvil
cp contracts-anvil.json contracts.json
echo "✅ Updated contracts.json to use Anvil addresses"

# Copy to public directory
cp contracts-anvil.json public/contracts.json
echo "✅ Copied to public/contracts.json"

# Copy to auth-server stores
cp contracts-anvil.json "$WORKSPACE_ROOT/packages/auth-server/stores/local-node.json"
echo "✅ Copied to packages/auth-server/stores/local-node.json"

# Copy to auth-server-api src
cp contracts-anvil.json "$WORKSPACE_ROOT/packages/auth-server-api/src/contracts.json"
echo "✅ Copied to packages/auth-server-api/src/contracts.json"

echo ""
echo "🎉 Deployment complete!"
echo ""
echo "📝 Contract Addresses (Anvil/Alto):"
cat contracts-anvil.json
