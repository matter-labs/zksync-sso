#!/bin/bash
set -e

# Configuration
RPC_URL="http://localhost:8456"
PRIVATE_KEY="0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"
DEPLOYER="0x36615Cf349d7F6344891B1e7CA7C72883F5dc049"
CONTRACTS_DIR="/home/colinbellmore/Documents/zksync-sso/packages/erc4337-contracts"

echo "🚀 Deploying MSA Factory and modules to ZKSync..."
echo ""
echo "📍 Deployer: $DEPLOYER"
echo "🌐 RPC URL: $RPC_URL"
echo ""

cd "$CONTRACTS_DIR"

# Deploy EOAKeyValidator implementation
echo "📦 Deploying EOAKeyValidator implementation..."
EOA_IMPL=$(forge create src/modules/EOAKeyValidator.sol:EOAKeyValidator \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync 2>&1 | grep "Deployed to:" | awk '{print $3}')
echo "✅ EOAKeyValidator implementation: $EOA_IMPL"

# Deploy TransparentUpgradeableProxy for EOAKeyValidator
echo ""
echo "📦 Deploying EOAKeyValidator proxy..."
EOA_VALIDATOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --constructor-args "$EOA_IMPL" "$DEPLOYER" "0x" 2>&1 | grep "Deployed to:" | awk '{print $3}')
echo "✅ EOAKeyValidator proxy: $EOA_VALIDATOR"

# Deploy SessionKeyValidator implementation
echo ""
echo "📦 Deploying SessionKeyValidator implementation..."
SESSION_IMPL=$(forge create src/modules/SessionKeyValidator.sol:SessionKeyValidator \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ SessionKeyValidator implementation: $SESSION_IMPL"

# Deploy proxy for SessionKeyValidator
echo ""
echo "📦 Deploying SessionKeyValidator proxy..."
SESSION_VALIDATOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --constructor-args "$SESSION_IMPL" "$DEPLOYER" "0x" \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ SessionKeyValidator proxy: $SESSION_VALIDATOR"

# Deploy WebAuthnValidator implementation
echo ""
echo "📦 Deploying WebAuthnValidator implementation..."
WEBAUTHN_IMPL=$(forge create src/modules/WebAuthnValidator.sol:WebAuthnValidator \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ WebAuthnValidator implementation: $WEBAUTHN_IMPL"

# Deploy proxy for WebAuthnValidator
echo ""
echo "📦 Deploying WebAuthnValidator proxy..."
WEBAUTHN_VALIDATOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --constructor-args "$WEBAUTHN_IMPL" "$DEPLOYER" "0x" \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ WebAuthnValidator proxy: $WEBAUTHN_VALIDATOR"

# Deploy GuardianExecutor implementation
echo ""
echo "📦 Deploying GuardianExecutor implementation..."
GUARDIAN_IMPL=$(forge create src/modules/GuardianExecutor.sol:GuardianExecutor \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ GuardianExecutor implementation: $GUARDIAN_IMPL"

# Deploy proxy for GuardianExecutor
echo ""
echo "📦 Deploying GuardianExecutor proxy..."
GUARDIAN_EXECUTOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --constructor-args "$GUARDIAN_IMPL" "$DEPLOYER" "0x" \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ GuardianExecutor proxy: $GUARDIAN_EXECUTOR"

# Deploy ModularSmartAccount implementation
echo ""
echo "📦 Deploying ModularSmartAccount implementation..."
ACCOUNT_IMPL=$(forge create src/ModularSmartAccount.sol:ModularSmartAccount \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ ModularSmartAccount implementation: $ACCOUNT_IMPL"

# Deploy UpgradeableBeacon
echo ""
echo "📦 Deploying UpgradeableBeacon..."
BEACON=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/beacon/UpgradeableBeacon.sol:UpgradeableBeacon \
  --constructor-args "$ACCOUNT_IMPL" "$DEPLOYER" \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ UpgradeableBeacon: $BEACON"

# Deploy MSAFactory implementation
echo ""
echo "📦 Deploying MSAFactory implementation..."
FACTORY_IMPL=$(forge create src/MSAFactory.sol:MSAFactory \
  --constructor-args "$BEACON" \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ MSAFactory implementation: $FACTORY_IMPL"

# Deploy proxy for MSAFactory
echo ""
echo "📦 Deploying MSAFactory proxy..."
FACTORY=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --constructor-args "$FACTORY_IMPL" "$DEPLOYER" "0x" \
  --private-key "$PRIVATE_KEY" \
  --rpc-url "$RPC_URL" \
  --zksync \
  --json 2>&1 | tail -1 | jq -r '.deployedTo')
echo "✅ MSAFactory proxy: $FACTORY"

# Create contracts.json
echo ""
echo "💾 Creating contracts.json..."
cd /home/colinbellmore/Documents/zksync-sso/examples/demo-app

cat > contracts.json << EOF
{
  "rpcUrl": "$RPC_URL",
  "chainId": 260,
  "deployer": "$DEPLOYER",
  "eoaValidator": "$EOA_VALIDATOR",
  "sessionValidator": "$SESSION_VALIDATOR",
  "webauthnValidator": "$WEBAUTHN_VALIDATOR",
  "guardianExecutor": "$GUARDIAN_EXECUTOR",
  "accountImplementation": "$ACCOUNT_IMPL",
  "beacon": "$BEACON",
  "factory": "$FACTORY"
}
EOF

echo "✅ Created contracts.json"

# Copy to public directory
cp contracts.json public/contracts.json
echo "✅ Copied to public/contracts.json"

echo ""
echo "🎉 Deployment complete!"
echo ""
echo "📝 Contract Addresses:"
cat contracts.json
