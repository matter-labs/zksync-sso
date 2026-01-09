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

# Update .env files with the deployed token address
cd ..

# Update both local and root .env files
for ENV_FILE in ".env" "../.env"; do
  # Get the absolute path for logging
  ABS_PATH=$(cd "$(dirname "$ENV_FILE")" && pwd)/$(basename "$ENV_FILE")

  # Create .env from .env.example if it doesn't exist
  if [ ! -f "$ENV_FILE" ]; then
    if [ -f "${ENV_FILE}.example" ]; then
      echo "Creating $ABS_PATH from .env.example..."
      cp "${ENV_FILE}.example" "$ENV_FILE"
    fi
  fi

  if [ -f "$ENV_FILE" ]; then
    # Remove existing VITE_TOKEN_ADDRESS line if present
    if grep -q "^VITE_TOKEN_ADDRESS=" "$ENV_FILE" || grep -q "^# VITE_TOKEN_ADDRESS=" "$ENV_FILE"; then
      # Use different sed syntax for macOS
      sed -i '' '/^VITE_TOKEN_ADDRESS=/d' "$ENV_FILE"
      sed -i '' '/^# VITE_TOKEN_ADDRESS=/d' "$ENV_FILE"
    fi

    # Append the new token address
    echo "VITE_TOKEN_ADDRESS=$CONTRACT_ADDRESS" >> "$ENV_FILE"
    echo "✓ Token address saved to $ABS_PATH"
  fi
done

echo ""
