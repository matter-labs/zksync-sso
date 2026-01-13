#!/bin/bash

# Dawn Mainnet Pre-Deployment Checklist
# Run this before deploying to ensure everything is ready

set -e

echo "🔍 Dawn Mainnet Pre-Deployment Checklist"
echo "========================================="
echo ""

ISSUES=0

# Check 1: Environment variables
echo "1. Checking environment variables..."
if [ -z "$WALLET_PRIVATE_KEY" ]; then
  echo "   ❌ WALLET_PRIVATE_KEY not set"
  ISSUES=$((ISSUES + 1))
else
  echo "   ✅ WALLET_PRIVATE_KEY set"
fi

if [ -z "$KEY_REGISTRY_OWNER_PRIVATE_KEY" ]; then
  echo "   ❌ KEY_REGISTRY_OWNER_PRIVATE_KEY not set"
  ISSUES=$((ISSUES + 1))
else
  echo "   ✅ KEY_REGISTRY_OWNER_PRIVATE_KEY set"
fi
echo ""

# Check 2: Cast CLI installed
echo "2. Checking for cast CLI..."
if command -v cast &> /dev/null; then
  echo "   ✅ cast installed ($(cast --version | head -n1))"
else
  echo "   ❌ cast not installed"
  echo "      Install with: curl -L https://foundry.paradigm.xyz | bash && foundryup"
  ISSUES=$((ISSUES + 1))
fi
echo ""

# Check 3: pnpm installed
echo "3. Checking for pnpm..."
if command -v pnpm &> /dev/null; then
  echo "   ✅ pnpm installed ($(pnpm --version))"
else
  echo "   ❌ pnpm not installed"
  echo "      Install with: npm install -g pnpm"
  ISSUES=$((ISSUES + 1))
fi
echo ""

# Check 4: Network connectivity
echo "4. Checking Dawn Mainnet connectivity..."
if curl -s -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  https://zksync-os-mainnet-dawn.zksync.io | grep -q "result"; then
  echo "   ✅ Dawn Mainnet RPC accessible"
else
  echo "   ❌ Cannot connect to Dawn Mainnet RPC"
  ISSUES=$((ISSUES + 1))
fi
echo ""

# Check 5: Deployer balance
if [ ! -z "$WALLET_PRIVATE_KEY" ]; then
  echo "5. Checking deployer wallet balance..."
  DEPLOYER_ADDRESS=$(cast wallet address --private-key $WALLET_PRIVATE_KEY 2>/dev/null || echo "")
  
  if [ ! -z "$DEPLOYER_ADDRESS" ]; then
    echo "   📍 Deployer address: $DEPLOYER_ADDRESS"
    
    BALANCE=$(cast balance $DEPLOYER_ADDRESS --rpc-url https://zksync-os-mainnet-dawn.zksync.io 2>/dev/null || echo "0")
    BALANCE_ETH=$(cast --to-unit $BALANCE ether 2>/dev/null || echo "0")
    
    echo "   💰 Balance: $BALANCE_ETH ETH"
    
    # Check if balance is sufficient (at least 0.5 ETH recommended)
    BALANCE_GWEI=$(cast --to-unit $BALANCE gwei 2>/dev/null | cut -d. -f1)
    if [ "$BALANCE_GWEI" -lt 500000000 ]; then
      echo "   ⚠️  Low balance. Recommended: at least 1 ETH"
      echo "      Transfer ETH to: $DEPLOYER_ADDRESS"
      ISSUES=$((ISSUES + 1))
    else
      echo "   ✅ Sufficient balance for deployment"
    fi
  else
    echo "   ❌ Invalid WALLET_PRIVATE_KEY"
    ISSUES=$((ISSUES + 1))
  fi
else
  echo "5. Skipping balance check (WALLET_PRIVATE_KEY not set)"
fi
echo ""

# Check 6: Key registry owner balance
if [ ! -z "$KEY_REGISTRY_OWNER_PRIVATE_KEY" ]; then
  echo "6. Checking key registry owner wallet balance..."
  OWNER_ADDRESS=$(cast wallet address --private-key $KEY_REGISTRY_OWNER_PRIVATE_KEY 2>/dev/null || echo "")
  
  if [ ! -z "$OWNER_ADDRESS" ]; then
    echo "   📍 Owner address: $OWNER_ADDRESS"
    
    BALANCE=$(cast balance $OWNER_ADDRESS --rpc-url https://zksync-os-mainnet-dawn.zksync.io 2>/dev/null || echo "0")
    BALANCE_ETH=$(cast --to-unit $BALANCE ether 2>/dev/null || echo "0")
    
    echo "   💰 Balance: $BALANCE_ETH ETH"
    
    # Check if balance is sufficient (at least 0.01 ETH recommended)
    BALANCE_GWEI=$(cast --to-unit $BALANCE gwei 2>/dev/null | cut -d. -f1)
    if [ "$BALANCE_GWEI" -lt 10000000 ]; then
      echo "   ⚠️  Low balance. Recommended: at least 0.1 ETH"
      echo "      Transfer ETH to: $OWNER_ADDRESS"
      ISSUES=$((ISSUES + 1))
    else
      echo "   ✅ Sufficient balance"
    fi
  else
    echo "   ❌ Invalid KEY_REGISTRY_OWNER_PRIVATE_KEY"
    ISSUES=$((ISSUES + 1))
  fi
else
  echo "6. Skipping owner balance check (KEY_REGISTRY_OWNER_PRIVATE_KEY not set)"
fi
echo ""

# Check 7: Contracts build
echo "7. Checking if contracts are built..."
if [ -d "packages/contracts/artifacts-zk" ]; then
  echo "   ✅ Contracts appear to be built"
else
  echo "   ⚠️  Contracts not built yet"
  echo "      Run: cd packages/contracts && pnpm build"
fi
echo ""

# Check 8: Dependencies installed
echo "8. Checking if dependencies are installed..."
if [ -d "node_modules" ] && [ -d "packages/contracts/node_modules" ]; then
  echo "   ✅ Dependencies installed"
else
  echo "   ⚠️  Dependencies not fully installed"
  echo "      Run: pnpm install -r"
fi
echo ""

# Summary
echo "========================================="
if [ $ISSUES -eq 0 ]; then
  echo "✅ All checks passed! Ready for deployment."
  echo ""
  echo "To deploy, run:"
  echo "  ./scripts/deploy-dawn-mainnet.sh"
  exit 0
else
  echo "❌ Found $ISSUES issue(s). Please fix them before deploying."
  echo ""
  echo "Need help? See: docs/DAWN_MAINNET_DEPLOYMENT.md"
  exit 1
fi
