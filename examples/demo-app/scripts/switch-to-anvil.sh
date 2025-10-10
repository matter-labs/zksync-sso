#!/bin/bash
set -e

echo "🔄 Switching Demo App to Anvil (port 8546)"
echo "=========================================="
echo ""
echo "Note: Using port 8546 because 8545 is occupied"
echo ""

# Check if Anvil is running on 8546
if nc -z 127.0.0.1 8546 2>/dev/null; then
  echo "⚠️  Port 8546 is already in use"
  echo ""
  echo "Checking if accounts are funded..."
  
  BALANCE=$(cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --rpc-url http://localhost:8546 2>/dev/null || echo "0")
  
  if [ "$BALANCE" = "0" ]; then
    echo "❌ Accounts have 0 balance - need to restart Anvil"
    echo ""
    echo "Please run the following commands:"
    echo ""
    echo "  # Kill existing process"
    echo "  pkill anvil"
    echo ""
    echo "  # Start Anvil with funded accounts"
    echo "  anvil --port 8546 --chain-id 9 --accounts 10 --balance 10000"
    echo ""
    echo "Then run this script again."
    exit 1
  else
    echo "✅ Accounts are funded (balance: $BALANCE wei)"
  fi
else
  echo "❌ Anvil is not running on port 8546"
  echo ""
  echo "Please start Anvil with:"
  echo "  anvil --port 8546 --chain-id 9 --accounts 10 --balance 10000"
  echo ""
  echo "Then run this script again."
  exit 1
fi

echo ""
echo "📦 Deploying MSA contracts to Anvil..."
echo ""

# Run the deployment script
./scripts/deploy-msa-anvil.sh

echo ""
echo "✅ Demo app is now configured for Anvil (port 8546)"
echo ""
echo "🚀 Next steps:"
echo "  1. Restart the demo-app: pnpm nx dev demo-app"
echo "  2. Visit: http://localhost:3003/web-sdk-test"
echo "  3. Click 'Deploy Account' to test"
echo ""
