#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CONTRACTS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${GREEN}Deploying NFT Quest contracts to Anvil...${NC}"

# Configuration
RPC_URL="http://127.0.0.1:8545"
ANVIL_ACCOUNT_0_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEPLOYER_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# Check if demo-app contracts exist
DEMO_APP_CONTRACTS="$PROJECT_ROOT/examples/demo-app/contracts-anvil.json"
if [ ! -f "$DEMO_APP_CONTRACTS" ]; then
    echo -e "${RED}Error: Demo-app contracts not found at $DEMO_APP_CONTRACTS${NC}"
    echo -e "${YELLOW}Please deploy demo-app's MSA factory first:${NC}"
    echo -e "  pnpm nx deploy-msa-factory demo-app"
    exit 1
fi

echo -e "${GREEN}Found demo-app contracts at $DEMO_APP_CONTRACTS${NC}"

# Deploy NFT Contract
echo -e "${GREEN}Deploying ZeekNFTQuest...${NC}"
BASE_TOKEN_URI="https://nft.zksync.dev/nft/metadata.json"

cd "$CONTRACTS_DIR"

DEPLOY_OUTPUT=$(forge create \
    --rpc-url "$RPC_URL" \
    --private-key "$ANVIL_ACCOUNT_0_KEY" \
    --broadcast \
    contracts/ZeekNFTQuest.sol:ZeekNFTQuest \
    --constructor-args "$BASE_TOKEN_URI" 2>&1)

NFT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | grep "Deployed to:" | awk '{print $3}')

echo -e "${GREEN}NFT Contract deployed at: $NFT_ADDRESS${NC}"

# Read shared infrastructure from demo-app
PAYMASTER=$(jq -r '.mockPaymaster // .testPaymaster' "$DEMO_APP_CONTRACTS")
SESSION_VALIDATOR=$(jq -r '.sessionValidator' "$DEMO_APP_CONTRACTS")
WEBAUTHN_VALIDATOR=$(jq -r '.webauthnValidator' "$DEMO_APP_CONTRACTS")
ENTRY_POINT=$(jq -r '.entryPoint' "$DEMO_APP_CONTRACTS")
BUNDLER_URL=$(jq -r '.bundlerUrl // "http://localhost:4337"' "$DEMO_APP_CONTRACTS")
FACTORY=$(jq -r '.factory' "$DEMO_APP_CONTRACTS")
EOA_VALIDATOR=$(jq -r '.eoaValidator' "$DEMO_APP_CONTRACTS")
GUARDIAN_EXECUTOR=$(jq -r '.guardianExecutor' "$DEMO_APP_CONTRACTS")

echo -e "${GREEN}Using shared ERC-4337 infrastructure from demo-app:${NC}"
echo -e "  Paymaster: $PAYMASTER"
echo -e "  Session Validator: $SESSION_VALIDATOR"
echo -e "  Webauthn Validator: $WEBAUTHN_VALIDATOR"
echo -e "  Entry Point: $ENTRY_POINT"
echo -e "  Bundler URL: $BUNDLER_URL"

# TODO: Replace MockPaymaster with a restricted paymaster that only allows NFT minting
# Current MockPaymaster allows all transactions - not production ready
echo -e "${YELLOW}WARNING: Using unrestricted MockPaymaster${NC}"
echo -e "${YELLOW}TODO: Implement restricted paymaster for NFT minting only${NC}"

# Write .env.local for Nuxt runtime config
ENV_FILE="$PROJECT_ROOT/examples/nft-quest/.env.local"
cat > "$ENV_FILE" << EOF
NUXT_PUBLIC_CONTRACTS_NFT=$NFT_ADDRESS
NUXT_PUBLIC_CONTRACTS_PAYMASTER=$PAYMASTER
EOF

echo -e "${GREEN}.env.local written to $ENV_FILE${NC}"

# Write public/contracts.json for runtime access
PUBLIC_DIR="$PROJECT_ROOT/examples/nft-quest/public"
mkdir -p "$PUBLIC_DIR"
CONTRACTS_JSON="$PUBLIC_DIR/contracts.json"

cat > "$CONTRACTS_JSON" << EOF
{
  "rpcUrl": "$RPC_URL",
  "chainId": 1337,
  "entryPoint": "$ENTRY_POINT",
  "bundlerUrl": "$BUNDLER_URL",
  "factory": "$FACTORY",
  "eoaValidator": "$EOA_VALIDATOR",
  "sessionValidator": "$SESSION_VALIDATOR",
  "webauthnValidator": "$WEBAUTHN_VALIDATOR",
  "guardianExecutor": "$GUARDIAN_EXECUTOR",
  "testPaymaster": "$PAYMASTER",
  "mockPaymaster": "$PAYMASTER",
  "nftContract": "$NFT_ADDRESS"
}
EOF

echo -e "${GREEN}contracts.json written to $CONTRACTS_JSON${NC}"

echo -e "${GREEN}✅ NFT-Quest deployment complete!${NC}"
echo -e "   - NFT Contract: $NFT_ADDRESS"
echo -e "   - Paymaster (shared): $PAYMASTER"
echo -e "   - Session Validator (shared): $SESSION_VALIDATOR"
