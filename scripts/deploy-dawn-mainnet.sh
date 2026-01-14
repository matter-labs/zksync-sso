#!/bin/bash

# Dawn Mainnet Quick Deploy Script
# This script provides a streamlined deployment process for Dawn Mainnet

set -e  # Exit on error

echo "🚀 Dawn Mainnet Deployment Script"
echo "=================================="
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if [ -z "$WALLET_PRIVATE_KEY" ]; then
  echo "❌ WALLET_PRIVATE_KEY environment variable not set"
  echo "   Set it with: export WALLET_PRIVATE_KEY=your_private_key"
  exit 1
fi

if [ -z "$KEY_REGISTRY_OWNER_PRIVATE_KEY" ]; then
  echo "❌ KEY_REGISTRY_OWNER_PRIVATE_KEY environment variable not set"
  echo "   Set it with: export KEY_REGISTRY_OWNER_PRIVATE_KEY=your_private_key"
  exit 1
fi

echo "✅ Environment variables set"
echo ""

# Get deployer address
DEPLOYER_ADDRESS=$(cast wallet address --private-key $WALLET_PRIVATE_KEY)
echo "📍 Deployer Address: $DEPLOYER_ADDRESS"

# Check deployer balance
BALANCE=$(cast balance $DEPLOYER_ADDRESS --rpc-url https://zksync-os-mainnet-dawn.zksync.io)
echo "💰 Deployer Balance: $(cast --to-unit $BALANCE ether) ETH"
echo ""

if [ "$(cast --to-unit $BALANCE ether | cut -d. -f1)" -lt 1 ]; then
  echo "⚠️  Warning: Low balance. Recommended: at least 1 ETH for deployment + paymaster funding"
  read -p "Continue anyway? (y/N): " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
  fi
fi

# Navigate to contracts directory
cd "$(dirname "$0")/../packages/contracts"

echo "🔨 Building contracts..."
pnpm build
echo "✅ Contracts built"
echo ""

# Ask for paymaster funding amount
read -p "💵 Enter amount of ETH to fund paymaster (e.g., 0.1, or 0 to skip): " FUND_AMOUNT
echo ""

# Deploy contracts
echo "🚀 Deploying contracts to Dawn Mainnet..."
pnpm hardhat deploy \
  --network dawnMainnet \
  --file ../auth-server/stores/dawn-mainnet.json \
  --fund $FUND_AMOUNT \
  --keyregistryowner $KEY_REGISTRY_OWNER_PRIVATE_KEY

echo "✅ Contracts deployed"
echo ""

# Verify deployment
echo "🔍 Verifying deployment..."
export WALLET_PRIVATE_KEY=$ORIGINAL_WALLET
node verify-dawn-deployment.js
echo ""

# Display next steps
echo "✅ Deployment Complete!"
echo ""
echo "📝 Next Steps:"
echo "1. Update packages/auth-server/stores/client.ts with deployed addresses"
echo "   (Addresses saved in: packages/auth-server/stores/dawn-mainnet.json)"
echo ""
echo "2. Deploy bundler service with the new contract addresses"
echo ""
echo "3. Deploy auth-server:"
echo "   cd packages/auth-server"
echo "   NUXT_PUBLIC_CHAIN_ID=30715 pnpm build"
echo ""
echo "4. Test the deployment end-to-end"
echo ""
echo "📄 Full deployment guide: docs/DAWN_MAINNET_DEPLOYMENT.md"
