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
CONTRACTS_FILE="$SCRIPT_DIR/../contracts-anvil.json"

echo "🚀 Deploying TestPaymaster to Anvil..."
echo ""
echo "📍 Deployer: $DEPLOYER_ADDRESS"
echo "🌐 RPC URL: $RPC_URL"
echo ""

cd "$CONTRACTS_DIR"

# Deploy TestPaymaster
echo "📦 Deploying TestPaymaster..."
PAYMASTER_OUTPUT=$(forge create \
  --rpc-url "$RPC_URL" \
  --private-key "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6" \
  src/test/TestPaymaster.sol:TestPaymaster 2>&1)

echo "$PAYMASTER_OUTPUT"

# Extract paymaster address
PAYMASTER_ADDRESS=$(echo "$PAYMASTER_OUTPUT" | grep "Deployed to:" | awk '{print $3}')

if [ -z "$PAYMASTER_ADDRESS" ]; then
  echo "❌ Failed to extract TestPaymaster address"
  exit 1
fi

echo ""
echo "✅ TestPaymaster deployed at: $PAYMASTER_ADDRESS"

# Fund the paymaster with 10 ETH from rich account (Anvil account #0)
echo ""
echo "💰 Funding paymaster with 10 ETH..."
cast send "$PAYMASTER_ADDRESS" \
  --value 10ether \
  --rpc-url "$RPC_URL" \
  --private-key "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

# Update contracts-anvil.json with paymaster address
echo ""
echo "📝 Updating contracts-anvil.json..."
jq ". + {\"testPaymaster\": \"$PAYMASTER_ADDRESS\"}" "$CONTRACTS_FILE" > "$CONTRACTS_FILE.tmp" && mv "$CONTRACTS_FILE.tmp" "$CONTRACTS_FILE"

echo ""
echo "✅ TestPaymaster setup complete!"
echo "   Address: $PAYMASTER_ADDRESS"
echo "   Balance: 10 ETH"
