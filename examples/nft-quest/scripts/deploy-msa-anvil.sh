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

# Use pnpm deploy-test to deploy all contracts
echo ""
echo "📦 Deploying contracts using pnpm deploy-test..."
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

# Verify all addresses were extracted
if [ -z "$EOA_VALIDATOR" ] || [ -z "$SESSION_VALIDATOR" ] || [ -z "$WEBAUTHN_VALIDATOR" ] || \
   [ -z "$GUARDIAN_EXECUTOR" ] || [ -z "$ACCOUNT_IMPL" ] || [ -z "$BEACON" ] || [ -z "$FACTORY" ]; then
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

# Create contracts-anvil.json
echo ""
echo "💾 Creating contracts-anvil.json..."
cd "$WORKSPACE_ROOT/examples/nft-quest"

cat > contracts-anvil.json << EOF
{
  "eoaValidator": "$EOA_VALIDATOR",
  "webauthnValidator": "$WEBAUTHN_VALIDATOR",
  "sessionValidator": "$SESSION_VALIDATOR",
  "factory": "$FACTORY",
  "bundlerUrl": "http://localhost:4337"
}
EOF

echo "✅ Created contracts-anvil.json"

# Copy to public directory for the Nuxt app
cp contracts-anvil.json public/contracts.json
echo "✅ Copied to public/contracts.json"

echo ""
echo "🎉 Deployment complete!"
echo ""
echo "📝 Contract Addresses (Anvil/Alto):"
cat contracts-anvil.json
