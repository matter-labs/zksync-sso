#!/bin/bash

# Quick USD token deployment script
# Usage: ./deploy-usd.sh [RPC_URL] [PRIVATE_KEY]

RPC_URL=${1:-http://localhost:3050}
PRIVATE_KEY=${2:-0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110}

echo "=== USD Token Deployment ==="
echo "RPC: $RPC_URL"
echo ""

cd token-deployment

# Compile contracts
echo "Compiling contracts..."
forge build

# Get bytecode and deploy
echo "Deploying TestUSD token..."
BYTECODE=$(forge inspect TestUSD bytecode)

RESULT=$(cast send --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  --create "$BYTECODE" \
  --json)

CONTRACT_ADDRESS=$(echo "$RESULT" | jq -r '.contractAddress')

echo ""
echo "✓ TestUSD deployed at: $CONTRACT_ADDRESS"
echo ""
echo "Token Details:"
echo "  Name: USD Test Token"
echo "  Symbol: USD"
echo "  Initial Supply: 1,000,000 USD"
echo ""
echo "Use this address for testing:"
echo "  $CONTRACT_ADDRESS"
