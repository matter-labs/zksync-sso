#!/bin/bash
set -e

echo "🔄 Switching Demo App to zkSync (port 8011)"
echo "=========================================="
echo ""

# Check if zkSync is running on 8011
if nc -z 127.0.0.1 8011 2>/dev/null; then
  echo "✅ zkSync node is running on port 8011"
else
  echo "❌ zkSync node is not running on port 8011"
  echo ""
  echo "Please start zkSync with:"
  echo "  anvil-zksync run"
  echo ""
  echo "Then run this script again."
  exit 1
fi

echo ""
echo "📦 Deploying MSA contracts to zkSync..."
echo ""

# Run the deployment script
./scripts/deploy-msa-zksync.sh

echo ""
echo "✅ Demo app is now configured for zkSync (port 8011)"
echo ""
echo "🚀 Next steps:"
echo "  1. Restart the demo-app: pnpm nx dev demo-app"
echo "  2. Visit: http://localhost:3003/web-sdk-test"
echo "  3. Click 'Deploy Account' to test"
echo ""
