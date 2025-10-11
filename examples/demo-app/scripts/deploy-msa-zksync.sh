#!/bin/bash
set -e

# Configuration
RPC_URL="http://localhost:8011"
PRIVATE_KEY="0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"
DEPLOYER="0x36615Cf349d7F6344891B1e7CA7C72883F5dc049"
CONTRACTS_DIR="/home/colinbellmore/Documents/zksync-sso/packages/erc4337-contracts"

echo "🚀 Deploying MSA Factory and modules to zkSync..."
echo ""
echo "📍 Deployer: $DEPLOYER"
echo "🌐 RPC URL: $RPC_URL"
echo ""

cd "$CONTRACTS_DIR"

# Helper function to extract deployed address from forge create output
extract_address() {
  grep "Deployed to:" | awk '{print $3}'
}

# Deploy EOAKeyValidator
echo "📦 [1/12] Deploying EOAKeyValidator implementation..."
EOA_IMPL=$(forge create src/modules/EOAKeyValidator.sol:EOAKeyValidator \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync 2>&1 | extract_address)
echo "✅ EOAKeyValidator impl: $EOA_IMPL"

echo "📦 [2/12] Deploying EOAKeyValidator proxy..."
EOA_VALIDATOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync --constructor-args "$EOA_IMPL" "$DEPLOYER" "0x" 2>&1 | extract_address)
echo "✅ EOAKeyValidator: $EOA_VALIDATOR"

# Deploy SessionKeyValidator
echo ""
echo "📦 [3/12] Deploying SessionKeyValidator implementation..."
SESSION_IMPL=$(forge create src/modules/SessionKeyValidator.sol:SessionKeyValidator \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync 2>&1 | extract_address)
echo "✅ SessionKeyValidator impl: $SESSION_IMPL"

echo "📦 [4/12] Deploying SessionKeyValidator proxy..."
SESSION_VALIDATOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync --constructor-args "$SESSION_IMPL" "$DEPLOYER" "0x" 2>&1 | extract_address)
echo "✅ SessionKeyValidator: $SESSION_VALIDATOR"

# Deploy WebAuthnValidator
echo ""
echo "📦 [5/12] Deploying WebAuthnValidator implementation..."
WEBAUTHN_IMPL=$(forge create src/modules/WebAuthnValidator.sol:WebAuthnValidator \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync 2>&1 | extract_address)
echo "✅ WebAuthnValidator impl: $WEBAUTHN_IMPL"

echo "📦 [6/12] Deploying WebAuthnValidator proxy..."
WEBAUTHN_VALIDATOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync --constructor-args "$WEBAUTHN_IMPL" "$DEPLOYER" "0x" 2>&1 | extract_address)
echo "✅ WebAuthnValidator: $WEBAUTHN_VALIDATOR"

# Deploy GuardianExecutor
echo ""
echo "📦 [7/12] Deploying GuardianExecutor implementation..."
GUARDIAN_IMPL=$(forge create src/modules/GuardianExecutor.sol:GuardianExecutor \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync 2>&1 | extract_address)
echo "✅ GuardianExecutor impl: $GUARDIAN_IMPL"

echo "📦 [8/12] Deploying GuardianExecutor proxy..."
GUARDIAN_EXECUTOR=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync --constructor-args "$GUARDIAN_IMPL" "$DEPLOYER" "0x" 2>&1 | extract_address)
echo "✅ GuardianExecutor: $GUARDIAN_EXECUTOR"

# Deploy ModularSmartAccount implementation
echo ""
echo "📦 [9/12] Deploying ModularSmartAccount implementation..."
ACCOUNT_IMPL=$(forge create src/ModularSmartAccount.sol:ModularSmartAccount \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync 2>&1 | extract_address)
echo "✅ ModularSmartAccount impl: $ACCOUNT_IMPL"

# Deploy UpgradeableBeacon
echo ""
echo "📦 [10/12] Deploying UpgradeableBeacon..."
BEACON=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/beacon/UpgradeableBeacon.sol:UpgradeableBeacon \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync --constructor-args "$ACCOUNT_IMPL" "$DEPLOYER" 2>&1 | extract_address)
echo "✅ UpgradeableBeacon: $BEACON"

# Deploy MSAFactory implementation
echo ""
echo "📦 [11/12] Deploying MSAFactory implementation..."
FACTORY_IMPL=$(forge create src/MSAFactory.sol:MSAFactory \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync --constructor-args "$BEACON" 2>&1 | extract_address)
echo "✅ MSAFactory impl: $FACTORY_IMPL"

# Deploy MSAFactory proxy
echo ""
echo "📦 [12/12] Deploying MSAFactory proxy..."
FACTORY=$(forge create dependencies/@openzeppelin-contracts-5.4.0/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy \
  --private-key "$PRIVATE_KEY" --rpc-url "$RPC_URL" --zksync --constructor-args "$FACTORY_IMPL" "$DEPLOYER" "0x" 2>&1 | extract_address)
echo "✅ MSAFactory: $FACTORY"

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
