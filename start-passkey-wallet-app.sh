#!/bin/bash

# =============================================================================
# ZKsync SSO Demo - Passkey Wallet App Startup Script
# =============================================================================
# This script starts all services needed for the passkey wallet demo:
# 1. L2-to-L1 Relayer (finalization daemon)
# 2. ERC-4337 Bundler
# 3. Passkey Wallet App
# 4. Optionally deploys the test USD token
#
# Usage:
#   ./start-passkey-wallet-app.sh                  # Start all services
#   ./start-passkey-wallet-app.sh --deploy-token   # Start all services and deploy token
#   ./start-passkey-wallet-app.sh --help           # Show help
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
DEPLOY_TOKEN=false
SHOW_HELP=false

for arg in "$@"; do
  case $arg in
    --deploy-token)
      DEPLOY_TOKEN=true
      shift
      ;;
    --help)
      SHOW_HELP=true
      shift
      ;;
    *)
      ;;
  esac
done

if [ "$SHOW_HELP" = true ]; then
  echo "Usage: ./start-passkey-wallet-app.sh [OPTIONS]"
  echo ""
  echo "Options:"
  echo "  --deploy-token    Deploy the test USD token to Chain A before starting services"
  echo "  --help            Show this help message"
  echo ""
  echo "This script will start:"
  echo "  1. L2-to-L1 Relayer (port 4340)"
  echo "  2. ERC-4337 Bundler (port 4337)"
  echo "  3. Passkey Wallet App (port 3000)"
  echo ""
  echo "Prerequisites:"
  echo "  - Copy .env.example to .env and fill in your values"
  echo "  - For interop features, ensure local chains are running on ports 3050 & 3051"
  exit 0
fi

# Function to print colored messages
print_info() {
  echo -e "${BLUE}ℹ${NC}  $1"
}

print_success() {
  echo -e "${GREEN}✓${NC}  $1"
}

print_error() {
  echo -e "${RED}✗${NC}  $1"
}

print_warning() {
  echo -e "${YELLOW}⚠${NC}  $1"
}

print_section() {
  echo ""
  echo -e "${BLUE}$1${NC}"
  echo "$(printf '=%.0s' {1..80})"
}

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

print_section "ZKsync SSO Demo - Starting All Services"

# Check if .env exists
if [ ! -f ".env" ]; then
  print_error ".env file not found!"
  echo ""
  echo "Please create a .env file from .env.example:"
  echo "  cp .env.example .env"
  echo ""
  echo "Then edit .env and fill in your configuration values."
  exit 1
fi

print_success "Found .env file"

# Load environment variables
set -a
source .env
set +a

# Check required variables
REQUIRED_VARS=(
  "DEPLOYER_PRIVATE_KEY"
  "L2_RPC_URL"
  "L1_RPC_URL"
)

MISSING_VARS=()
for var in "${REQUIRED_VARS[@]}"; do
  if [ -z "${!var}" ] || [ "${!var}" = "0xYOUR_PRIVATE_KEY_HERE" ] || [[ "${!var}" == *"YOUR_ALCHEMY_KEY_HERE"* ]]; then
    MISSING_VARS+=("$var")
  fi
done

if [ ${#MISSING_VARS[@]} -gt 0 ]; then
  print_error "Missing or invalid required environment variables:"
  for var in "${MISSING_VARS[@]}"; do
    echo "  - $var"
  done
  echo ""
  echo "Please update your .env file with valid values."
  exit 1
fi

print_success "Environment variables validated"

# Create log directory
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
print_success "Log directory: $LOG_DIR"

# Deploy token if requested
if [ "$DEPLOY_TOKEN" = true ]; then
  print_section "Deploying Test USD Token"

  if [ ! -d "passkey-wallet-app/token-deployment" ]; then
    print_error "Token deployment directory not found"
    exit 1
  fi

  print_info "Deploying token to Chain A ($CHAIN_A_RPC)..."
  cd passkey-wallet-app
  ./deploy-usd.sh "$CHAIN_A_RPC" "$INTEROP_PRIVATE_KEY"

  if [ $? -eq 0 ]; then
    print_success "Token deployed successfully"
    # Reload .env to get VITE_TOKEN_ADDRESS
    set -a
    source ../.env
    set +a
  else
    print_error "Token deployment failed"
    exit 1
  fi

  cd "$SCRIPT_DIR"
fi

# Function to stop all background processes
cleanup() {
  print_section "Shutting Down Services"

  if [ ! -z "$RELAYER_PID" ] && ps -p $RELAYER_PID > /dev/null 2>&1; then
    print_info "Stopping L2-to-L1 Relayer (PID: $RELAYER_PID)..."
    kill $RELAYER_PID 2>/dev/null || true
  fi

  if [ ! -z "$BUNDLER_PID" ] && ps -p $BUNDLER_PID > /dev/null 2>&1; then
    print_info "Stopping Bundler (PID: $BUNDLER_PID)..."
    kill $BUNDLER_PID 2>/dev/null || true
  fi

  if [ ! -z "$WALLET_PID" ] && ps -p $WALLET_PID > /dev/null 2>&1; then
    print_info "Stopping Wallet App (PID: $WALLET_PID)..."
    kill $WALLET_PID 2>/dev/null || true
  fi

  print_success "All services stopped"
  exit 0
}

# Register cleanup function
trap cleanup SIGINT SIGTERM

# Start L2-to-L1 Relayer
print_section "Starting L2-to-L1 Relayer"
cd "$SCRIPT_DIR/l2-to-l1-relayer"

# Create .env for relayer
cat > .env << EOF
EXECUTOR_PRIVATE_KEY=$DEPLOYER_PRIVATE_KEY
L2_RPC_URL=$L2_RPC_URL
L1_RPC_URL=$L1_RPC_URL
L2_INTEROP_CENTER=$L2_INTEROP_CENTER
L1_INTEROP_HANDLER=$L1_INTEROP_HANDLER
POLL_INTERVAL=${POLL_INTERVAL:-30000}
FINALIZATION_WAIT=${FINALIZATION_WAIT:-900000}
EOF

print_info "Starting relayer daemon..."
node auto-finalize-daemon.js > "$LOG_DIR/relayer.log" 2>&1 &
RELAYER_PID=$!
print_success "L2-to-L1 Relayer started (PID: $RELAYER_PID, Port: ${RELAYER_STATUS_PORT:-4340})"
print_info "Logs: $LOG_DIR/relayer.log"

# Wait a moment for relayer to start
sleep 2

# Start ERC-4337 Bundler
print_section "Starting ERC-4337 Bundler"
cd "$SCRIPT_DIR/packages/bundler"

# Create .env for bundler
cat > .env << EOF
EXECUTOR_PRIVATE_KEY=$DEPLOYER_PRIVATE_KEY
UTILITY_PRIVATE_KEY=$DEPLOYER_PRIVATE_KEY
RPC_URL=$L2_RPC_URL
EOF

# Update alto-config.json with private keys from .env
print_info "Updating alto-config.json with private keys..."
if [ -f "alto-config.json" ]; then
  # Create temporary config with updated values
  cat alto-config.json | \
    jq --arg pk "$DEPLOYER_PRIVATE_KEY" '.["executor-private-keys"] = $pk' | \
    jq --arg pk "$DEPLOYER_PRIVATE_KEY" '.["utility-private-key"] = $pk' | \
    jq --arg rpc "$L2_RPC_URL" '.["rpc-url"] = $rpc' | \
    jq --argjson port "${BUNDLER_PORT:-4337}" '.port = $port' \
    > alto-config.json.tmp
  mv alto-config.json.tmp alto-config.json
  print_success "alto-config.json updated"
else
  print_warning "alto-config.json not found, skipping update"
fi

print_info "Installing bundler dependencies (if needed)..."
npm install --silent > /dev/null 2>&1 || true

print_info "Building bundler..."
npm run build > "$LOG_DIR/bundler-build.log" 2>&1
if [ $? -ne 0 ]; then
  print_error "Bundler build failed. Check $LOG_DIR/bundler-build.log"
  exit 1
fi
print_success "Bundler built successfully"

print_info "Starting bundler..."
npm start > "$LOG_DIR/bundler.log" 2>&1 &
BUNDLER_PID=$!
print_success "ERC-4337 Bundler started (PID: $BUNDLER_PID, Port: ${BUNDLER_PORT:-4337})"
print_info "Logs: $LOG_DIR/bundler.log"

# Wait a moment for bundler to start
sleep 2

# Start Passkey Wallet App
print_section "Starting Passkey Wallet App"
cd "$SCRIPT_DIR/passkey-wallet-app"

# Create .env for wallet app
cat > .env << EOF
VITE_DEPLOYER_PRIVATE_KEY=$DEPLOYER_PRIVATE_KEY
VITE_INTEROP_PRIVATE_KEY=$INTEROP_PRIVATE_KEY
EOF

# Add token address if it exists
if [ ! -z "$VITE_TOKEN_ADDRESS" ]; then
  echo "VITE_TOKEN_ADDRESS=$VITE_TOKEN_ADDRESS" >> .env
fi

print_info "Installing wallet app dependencies (if needed)..."
npm install --silent > /dev/null 2>&1 || true

print_info "Starting wallet app..."
npm run dev > "$LOG_DIR/wallet.log" 2>&1 &
WALLET_PID=$!
print_success "Passkey Wallet App started (PID: $WALLET_PID, Port: ${WALLET_APP_PORT:-3000})"
print_info "Logs: $LOG_DIR/wallet.log"

# Wait for services to fully start
print_section "Waiting for Services to Start"
sleep 5

# Summary
print_section "All Services Running"
echo ""
echo "Services:"
echo "  • L2-to-L1 Relayer    → http://localhost:${RELAYER_STATUS_PORT:-4340}/status"
echo "  • ERC-4337 Bundler    → http://localhost:${BUNDLER_PORT:-4337}"
echo "  • Passkey Wallet App  → http://localhost:${WALLET_APP_PORT:-3000}"
echo ""
echo "Process IDs:"
echo "  • Relayer: $RELAYER_PID"
echo "  • Bundler: $BUNDLER_PID"
echo "  • Wallet:  $WALLET_PID"
echo ""
echo "Logs:"
echo "  • Relayer: $LOG_DIR/relayer.log"
echo "  • Bundler: $LOG_DIR/bundler.log"
echo "  • Wallet:  $LOG_DIR/wallet.log"
echo ""
print_info "Press Ctrl+C to stop all services"
echo ""

# Wait for user interrupt
wait
