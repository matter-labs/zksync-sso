#!/bin/bash

echo "🤖 Starting L2-to-L1 Relayer Service..."
echo ""

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo ""
fi

# Get executor address
EXECUTOR_ADDRESS=$(node -e "
import { privateKeyToAccount } from 'viem/accounts';
const account = privateKeyToAccount('${EXECUTOR_PRIVATE_KEY:-0x1cfcab2cf5ad255cb3387f7fdca2651a61377b334a2a3daa4af86eb476369105}');
console.log(account.address);
")

echo "💰 Executor Address: $EXECUTOR_ADDRESS"
echo ""
echo "⚠️  Make sure this address has Sepolia ETH!"
echo "   Faucet: https://www.alchemy.com/faucets/ethereum-sepolia"
echo ""
echo "Press Ctrl+C to stop the relayer"
echo ""

# Start the service
npm start
