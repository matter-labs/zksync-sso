#!/bin/bash

# Deploy MSA Factory and modules for demo-app
# This script uses forge to deploy all contracts and saves the addresses

set -e

echo "🚀 Deploying MSA Factory and modules..."
echo ""

# Configuration
DEPLOYER_KEY="0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"
RPC_URL="http://localhost:8456"
CHAIN_ID="260"
ERC4337_CONTRACTS_PATH="../../packages/erc4337-contracts"

echo "📍 Deployer: $(cast wallet address $DEPLOYER_KEY)"
echo "🌐 RPC URL: $RPC_URL"
echo "⛓️  Chain ID: $CHAIN_ID"
echo ""

# Check balance
BALANCE=$(cast balance $(cast wallet address $DEPLOYER_KEY) --rpc-url $RPC_URL)
echo "💰 Deployer balance: $(cast --to-unit $BALANCE ether) ETH"
echo ""

if [ "$BALANCE" = "0" ]; then
  echo "❌ Deployer has no balance. Please fund the account."
  exit 1
fi

# Deploy contracts using forge script
echo "📦 Deploying contracts..."
echo ""

cd $ERC4337_CONTRACTS_PATH

# Run the forge deployment script with ZKSync flag
echo "Deploying MSA Factory and modules..."
PRIVATE_KEY="$PRIVATE_KEY" forge script "$SCRIPT_PATH" \
  --sig 'run()' \
  --rpc-url "$RPC_URL" \
  --broadcast \
  --zksync

FORGE_EXIT_CODE=$?
set -e

cd -

# Check if broadcast file was created (deployment succeeded even if send failed)
BROADCAST_FILE="$ERC4337_CONTRACTS_PATH/broadcast/Deploy.s.sol/$CHAIN_ID/run-latest.json"

if [ ! -f "$BROADCAST_FILE" ]; then
  echo ""
  echo "❌ Deployment failed - broadcast file not found: $BROADCAST_FILE"
  cat /tmp/forge-deploy.log
  exit 1
fi

echo ""
if [ $FORGE_EXIT_CODE -ne 0 ]; then
  echo "⚠️  Some transactions may have failed, but contracts were broadcast. Continuing..."
else
  echo "✅ Deployment complete!"
fi
echo ""

# Parse the broadcast file to get deployed addresses
echo "📖 Parsing deployment addresses..."

# Extract addresses using jq
# Transaction order: [impl, proxy, impl, proxy, impl, proxy, impl, proxy, impl, beacon, impl, proxy]
# We want the proxies for validators and the final proxy for factory
EOA_VALIDATOR=$(jq -r '.transactions[1].contractAddress' $BROADCAST_FILE)
SESSION_VALIDATOR=$(jq -r '.transactions[3].contractAddress' $BROADCAST_FILE)
WEBAUTHN_VALIDATOR=$(jq -r '.transactions[5].contractAddress' $BROADCAST_FILE)
GUARDIAN_EXECUTOR=$(jq -r '.transactions[7].contractAddress' $BROADCAST_FILE)
ACCOUNT_IMPL=$(jq -r '.transactions[8].contractAddress' $BROADCAST_FILE)
BEACON=$(jq -r '.transactions[9].contractAddress' $BROADCAST_FILE)
FACTORY=$(jq -r '.transactions[11].contractAddress' $BROADCAST_FILE)

# Create contracts.json
cat > contracts.json <<EOF
{
  "eoaValidator": "$EOA_VALIDATOR",
  "sessionValidator": "$SESSION_VALIDATOR",
  "webauthnValidator": "$WEBAUTHN_VALIDATOR",
  "guardianExecutor": "$GUARDIAN_EXECUTOR",
  "accountImplementation": "$ACCOUNT_IMPL",
  "beacon": "$BEACON",
  "factory": "$FACTORY",
  "chainId": $CHAIN_ID,
  "rpcUrl": "$RPC_URL",
  "deployedAt": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

# Also copy to public directory so it can be served
cp contracts.json public/contracts.json

echo "✅ Contract addresses saved to contracts.json and public/contracts.json"
echo ""
echo "📋 Deployed Contracts:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "EOAKeyValidator:        $EOA_VALIDATOR"
echo "SessionKeyValidator:    $SESSION_VALIDATOR"
echo "WebAuthnValidator:      $WEBAUTHN_VALIDATOR"
echo "GuardianExecutor:       $GUARDIAN_EXECUTOR"
echo "Account Implementation: $ACCOUNT_IMPL"
echo "UpgradeableBeacon:      $BEACON"
echo "MSAFactory:             $FACTORY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎉 Deployment complete!"
echo ""
echo "💡 The factory address can now be used in the demo-app:"
echo "   Factory: $FACTORY"
echo ""
