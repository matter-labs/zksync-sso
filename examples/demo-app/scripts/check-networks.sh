#!/bin/bash

# MSA Network Status Checker
# Checks which networks are ready for deployment

echo "🔍 MSA Network Status Checker"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check zkSync (port 8011)
echo "📡 zkSync Network (port 8011)"
echo "--------------------------------"

if curl -s -X POST http://localhost:8011 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' &>/dev/null; then
  
  ZKSYNC_CHAIN_ID=$(cast chain-id --rpc-url http://localhost:8011 2>/dev/null)
  
  if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ zkSync node is running${NC}"
    echo "   Chain ID: $ZKSYNC_CHAIN_ID"
    
    # Check if contracts are deployed
    if [ -f "contracts-zksync.json" ]; then
      FACTORY=$(jq -r '.factory' contracts-zksync.json)
      if [ "$FACTORY" != "null" ] && [ "$FACTORY" != "" ]; then
        echo -e "${GREEN}✅ Contracts deployed${NC}"
        echo "   Factory: $FACTORY"
        
        # Check if factory has code
        CODE=$(cast code "$FACTORY" --rpc-url http://localhost:8011 2>/dev/null | wc -c)
        if [ "$CODE" -gt 10 ]; then
          echo -e "${GREEN}✅ Factory verified on-chain ($CODE bytes)${NC}"
        else
          echo -e "${RED}❌ Factory address has no code${NC}"
        fi
      else
        echo -e "${YELLOW}⚠️  No contracts deployed yet${NC}"
      fi
    else
      echo -e "${YELLOW}⚠️  No contracts-zksync.json found${NC}"
    fi
  else
    echo -e "${RED}❌ zkSync node not responding${NC}"
  fi
else
  echo -e "${RED}❌ zkSync node not running${NC}"
  echo "   Start with: anvil-zksync run"
fi

echo ""

# Check Anvil (port 8546)
echo "📡 Anvil Network (port 8546)"
echo "--------------------------------"

if curl -s -X POST http://localhost:8546 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' &>/dev/null; then
  
  ANVIL_CHAIN_ID=$(cast chain-id --rpc-url http://localhost:8546 2>/dev/null)
  
  if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Anvil node is running${NC}"
    echo "   Chain ID: $ANVIL_CHAIN_ID"
    
    # Check default account balance
    BALANCE=$(cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --rpc-url http://localhost:8546 2>/dev/null)
    
    if [ $? -eq 0 ]; then
      if [ "$BALANCE" != "0" ]; then
        BALANCE_ETH=$(echo "scale=2; $BALANCE / 1000000000000000000" | bc)
        echo -e "${GREEN}✅ Deployer account funded${NC}"
        echo "   Balance: ${BALANCE_ETH} ETH"
      else
        echo -e "${RED}❌ Deployer account has 0 balance${NC}"
        echo -e "${YELLOW}   Restart Anvil with: anvil --port 8546 --chain-id 9 --accounts 10 --balance 10000${NC}"
      fi
    fi
    
    # Check if contracts are deployed
    if [ -f "contracts-anvil.json" ]; then
      FACTORY=$(jq -r '.factory' contracts-anvil.json)
      if [ "$FACTORY" != "null" ] && [ "$FACTORY" != "" ]; then
        echo -e "${GREEN}✅ Contracts deployed${NC}"
        echo "   Factory: $FACTORY"
        
        # Check if factory has code
        CODE=$(cast code "$FACTORY" --rpc-url http://localhost:8546 2>/dev/null | wc -c)
        if [ "$CODE" -gt 10 ]; then
          echo -e "${GREEN}✅ Factory verified on-chain ($CODE bytes)${NC}"
        else
          echo -e "${RED}❌ Factory address has no code${NC}"
        fi
      else
        echo -e "${YELLOW}⚠️  No contracts deployed yet${NC}"
      fi
    else
      echo -e "${YELLOW}⚠️  No contracts-anvil.json found${NC}"
    fi
  else
    echo -e "${RED}❌ Anvil node not responding${NC}"
  fi
else
  echo -e "${RED}❌ Anvil node not running${NC}"
  echo "   Start with: anvil --port 8546 --chain-id 9 --accounts 10 --balance 10000"
fi

echo ""

# Check current demo-app configuration
echo "📱 Demo App Configuration"
echo "--------------------------------"

if [ -f "public/contracts.json" ]; then
  RPC_URL=$(jq -r '.rpcUrl' public/contracts.json)
  CHAIN_ID=$(jq -r '.chainId' public/contracts.json)
  FACTORY=$(jq -r '.factory' public/contracts.json)
  
  echo "   RPC URL: $RPC_URL"
  echo "   Chain ID: $CHAIN_ID"
  echo "   Factory: $FACTORY"
  
  if [ "$CHAIN_ID" = "260" ]; then
    echo -e "${GREEN}   ✅ Configured for zkSync${NC}"
  elif [ "$CHAIN_ID" = "9" ] || [ "$CHAIN_ID" = "31337" ]; then
    echo -e "${GREEN}   ✅ Configured for Anvil${NC}"
  else
    echo -e "${YELLOW}   ⚠️  Unknown chain ID${NC}"
  fi
else
  echo -e "${RED}❌ No public/contracts.json found${NC}"
  echo "   Run deployment script first"
fi

echo ""
echo "================================"
echo ""

# Summary and recommendations
echo "💡 Recommendations:"
echo ""

# Check zkSync status
if curl -s -X POST http://localhost:8011 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' &>/dev/null && \
  [ -f "contracts-zksync.json" ]; then
  echo -e "${GREEN}✅ zkSync is ready to use${NC}"
else
  echo "For zkSync:"
  echo "  1. Start: anvil-zksync run"
  echo "  2. Deploy: ./scripts/deploy-msa-zksync.sh"
fi

echo ""

# Check Anvil status
if curl -s -X POST http://localhost:8546 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' &>/dev/null; then
  
  BALANCE=$(cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --rpc-url http://localhost:8546 2>/dev/null)
  
  if [ "$BALANCE" = "0" ] || [ -z "$BALANCE" ]; then
    echo "For Anvil:"
    echo "  1. Kill existing: pkill anvil"
    echo "  2. Start: anvil --port 8546 --chain-id 9 --accounts 10 --balance 10000"
    echo "  3. Deploy: ./scripts/deploy-msa-anvil.sh"
  elif [ -f "contracts-anvil.json" ]; then
    echo -e "${GREEN}✅ Anvil is ready to use${NC}"
  else
    echo "For Anvil:"
    echo "  1. Deploy: ./scripts/deploy-msa-anvil.sh"
  fi
else
  echo "For Anvil:"
  echo "  1. Start: anvil --port 8546 --chain-id 9 --accounts 10 --balance 10000"
  echo "  2. Deploy: ./scripts/deploy-msa-anvil.sh"
fi

echo ""
