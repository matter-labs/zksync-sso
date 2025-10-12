#!/bin/bash
set -e

# Configuration
RPC_URL="http://localhost:8456"
PRIVATE_KEY="0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"
DEPLOYER="0x36615Cf349d7F6344891B1e7CA7C72883F5dc049"

echo "🚀 Deploying MSA Factory with cast send..."
echo "📍 Deployer: $DEPLOYER"
echo "🌐 RPC URL: $RPC_URL"

# Get contract artifacts
CONTRACTS_DIR="/home/colinbellmore/Documents/zksync-sso/packages/erc4337-contracts"

# Deploy EOAKeyValidator implementation
echo ""
echo "📦 Deploying EOAKeyValidator implementation..."
EOA_IMPL_BYTECODE=$(jq -r '.bytecode.object' "$CONTRACTS_DIR/out/EOAKeyValidator.sol/EOAKeyValidator.json")
EOA_IMPL=$(cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" --create "$EOA_IMPL_BYTECODE" --json | jq -r '.contractAddress')
echo "✅ EOAKeyValidator implementation: $EOA_IMPL"

# Deploy EOAKeyValidator proxy
echo ""
echo "📦 Deploying EOAKeyValidator proxy..."
PROXY_BYTECODE=$(jq -r '.bytecode.object' "$CONTRACTS_DIR/out/TransparentUpgradeableProxy.sol/TransparentUpgradeableProxy.json")
# Constructor: address logic, address admin, bytes memory data
# We'll use deployer as admin for now
PROXY_CONSTRUCTOR=$(cast abi-encode "constructor(address,address,bytes)" "$EOA_IMPL" "$DEPLOYER" "0x")
EOA_VALIDATOR=$(cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" --create "${PROXY_BYTECODE}${PROXY_CONSTRUCTOR:2}" --json | jq -r '.contractAddress')
echo "✅ EOAKeyValidator proxy: $EOA_VALIDATOR"

echo ""
echo "🎉 Deployment complete!"
echo "EOAKeyValidator: $EOA_VALIDATOR"
