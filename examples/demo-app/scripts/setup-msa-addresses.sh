#!/bin/bash
set -e

# Configuration
RPC_URL="http://localhost:8456"
DEPLOYER="0x36615Cf349d7F6344891B1e7CA7C72883F5dc049"

echo "🚀 Setting up MSA Factory addresses for demo-app..."
echo ""
echo "📍 Deployer: $DEPLOYER"
echo "🌐 RPC URL: $RPC_URL"
echo ""
echo "⚠️  Using mock addresses for local testing"
echo "   Run actual deployment manually with: cd packages/erc4337-contracts && forge script script/Deploy.s.sol"
echo ""

# Create contracts.json with mock addresses (or use addresses from a previous deployment)
# These addresses are deterministic CREATE2 addresses that will be the same each time
cat > contracts.json << EOF
{
  "rpcUrl": "$RPC_URL",
  "chainId": 260,
  "deployer": "$DEPLOYER",
  "eoaValidator": "0x38Bf206f027B9c861643689CD516A3B00210586f",
  "sessionValidator": "0xD52c9b1bA249f877C8492F64c096E37a8072982A",
  "webauthnValidator": "0x4337768cB3eC57Dd2cb843eFb929B773B13322de",
  "guardianExecutor": "0x1dEedcF23b6970C30B9d82e18F27B0844bF37838",
  "accountImplementation": "0x689679B8c5559b842e30Fa0D817A3F4Ca09FE726",
  "beacon": "0x121d7fB7D7B28eBcCf017A8175b8DD637C670BBc",
  "factory": "0x53FD3d1028481AFeC9c55be22135f9EC6Bb9eCC0"
}
EOF

echo "✅ Created contracts.json"

# Copy to public directory
cp contracts.json public/contracts.json
echo "✅ Copied to public/contracts.json"

echo ""
echo "📝 Contract Addresses:"
cat contracts.json

echo ""
echo "✅ Setup complete! The demo-app can now use these addresses."
echo ""
echo "Note: These are the predicted CREATE2 addresses from Forge."
echo "To actually deploy the contracts, you'll need to run the Forge deployment"
echo "with ZKSync-compatible transaction formatting."
