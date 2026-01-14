# ZKsync SSO Demo - Setup Guide

This guide provides instructions for setting up and running the ZKsync SSO Demo
application.

## Table of Contents

- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Configuration](#configuration)
- [Running the Application](#running-the-application)
  - [Option 1: All-in-One (Recommended)](#option-1-all-in-one-recommended)
  - [Option 2: Individual Services](#option-2-individual-services)
- [Token Deployment](#token-deployment)
- [Interop Testing](#interop-testing)
- [Troubleshooting](#troubleshooting)

## Quick Start

The fastest way to get started:

```bash
# 1. Configure environment
cp .env.example .env
# Edit .env and fill in your values (see Configuration section)

# 2. Start all services (with token deployment)
./start-passkey-wallet-app.sh --deploy-token
```

That's it! The wallet app will be available at <http://localhost:3000>

## Prerequisites

### Required Software

- **Node.js** (v18 or later)
- **npm** (comes with Node.js)
- **Foundry** (for token deployment)

  ```bash
  curl -L https://foundry.paradigm.xyz | bash
  foundryup
  ```

- **jq** (for JSON parsing)

  ```bash
  # macOS
  brew install jq

  # Ubuntu/Debian
  sudo apt-get install jq
  ```

### Required Accounts & Keys

1. **Private Key with Funds**

   - You need a private key with ETH on:

     - **L1 (Sepolia)** - for L2-to-L1 finalization
     - **L2 (ZKsync OS Testnet)** - for bundler operations

   - Get Sepolia ETH from: <https://sepoliafaucet.com/>
   - Get ZKsync OS Testnet ETH from: <https://portal.zksync.io/bridge/>

2. **Alchemy API Key**
   - Sign up at: <https://www.alchemy.com/>
   - Create a new app on Sepolia testnet
   - Copy your API key

### Optional (for Interop Testing)

- **Local ZKsync Chains** running on ports 3050 and 3051
- See [Interop Testing](#interop-testing) section for setup

## Configuration

### 1. Create Configuration File

Copy the example configuration:

```bash
cp .env.example .env
```

### 2. Edit Configuration

Open `.env` in your editor and fill in the required values:

```bash
# =============================================================================
# REQUIRED CONFIGURATION
# =============================================================================

# Your private key (must have ETH on both L1 Sepolia and L2 ZKsync OS)
DEPLOYER_PRIVATE_KEY=0xYOUR_ACTUAL_PRIVATE_KEY_HERE

# Your Alchemy API key for Sepolia
L1_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_ACTUAL_ALCHEMY_KEY_HERE

# =============================================================================
# OPTIONAL CONFIGURATION (defaults are usually fine)
# =============================================================================

# L2 RPC URL (default works for most users)
L2_RPC_URL=https://zksync-os-testnet-alpha.zksync.dev/

# Interop private key (only needed for local chain testing)
INTEROP_PRIVATE_KEY=0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110

# Service ports (change if you have conflicts)
BUNDLER_PORT=4337
RELAYER_STATUS_PORT=4340
WALLET_APP_PORT=3000
```

## Running the Application

### Option 1: All-in-One (Recommended)

Use the unified start script to launch all services at once:

#### Start Everything

```bash
./start-passkey-wallet-app.sh
```

This starts:

- L2-to-L1 Relayer (finalization daemon)
- ERC-4337 Bundler
- Passkey Wallet App

#### Start with Token Deployment

```bash
./start-passkey-wallet-app.sh --deploy-token
```

This does everything above PLUS deploys the test USD token to Chain A.

#### View Help

```bash
./start-passkey-wallet-app.sh --help
```

#### Stop All Services

Press `Ctrl+C` in the terminal where you ran `start-passkey-wallet-app.sh`

#### View Logs

Logs are saved to the `logs/` directory:

```bash
# Follow relayer logs
tail -f logs/relayer.log

# Follow bundler logs
tail -f logs/bundler.log

# Follow wallet app logs
tail -f logs/wallet.log
```

### Option 2: Individual Services

If you prefer to run services separately (useful for debugging):

#### Terminal 1: L2-to-L1 Relayer

```bash
cd l2-to-l1-relayer

# Create .env from root config
cat > .env << EOF
EXECUTOR_PRIVATE_KEY=$(grep DEPLOYER_PRIVATE_KEY ../.env | cut -d= -f2)
L2_RPC_URL=$(grep L2_RPC_URL ../.env | cut -d= -f2)
L1_RPC_URL=$(grep L1_RPC_URL ../.env | cut -d= -f2)
L2_INTEROP_CENTER=$(grep L2_INTEROP_CENTER ../.env | cut -d= -f2)
L1_INTEROP_HANDLER=$(grep L1_INTEROP_HANDLER ../.env | cut -d= -f2)
POLL_INTERVAL=$(grep POLL_INTERVAL ../.env | cut -d= -f2)
FINALIZATION_WAIT=$(grep FINALIZATION_WAIT ../.env | cut -d= -f2)
EOF

# Start relayer
node auto-finalize-daemon.js
```

#### Terminal 2: ERC-4337 Bundler

```bash
cd packages/bundler

# Create .env from root config
cat > .env << EOF
EXECUTOR_PRIVATE_KEY=$(grep DEPLOYER_PRIVATE_KEY ../../.env | cut -d= -f2)
UTILITY_PRIVATE_KEY=$(grep DEPLOYER_PRIVATE_KEY ../../.env | cut -d= -f2)
RPC_URL=$(grep L2_RPC_URL ../../.env | cut -d= -f2)
EOF

# Update alto-config.json with private keys
DEPLOYER_KEY=$(grep DEPLOYER_PRIVATE_KEY ../../.env | cut -d= -f2)
L2_RPC=$(grep L2_RPC_URL ../../.env | cut -d= -f2)
BUNDLER_PORT=$(grep BUNDLER_PORT ../../.env | cut -d= -f2)
BUNDLER_PORT=${BUNDLER_PORT:-4337}

cat alto-config.json | \
  jq --arg pk "$DEPLOYER_KEY" '.["executor-private-keys"] = $pk' | \
  jq --arg pk "$DEPLOYER_KEY" '.["utility-private-key"] = $pk' | \
  jq --arg rpc "$L2_RPC" '.["rpc-url"] = $rpc' | \
  jq --argjson port "$BUNDLER_PORT" '.port = $port' \
  > alto-config.json.tmp
mv alto-config.json.tmp alto-config.json

# Install dependencies (first time only)
npm install

# Build bundler (required before first run)
npm run build

# Start bundler
npm start
```

#### Terminal 3: Passkey Wallet App

```bash
cd passkey-wallet-app

# Create .env from root config
cat > .env << EOF
VITE_DEPLOYER_PRIVATE_KEY=$(grep DEPLOYER_PRIVATE_KEY ../.env | cut -d= -f2)
VITE_INTEROP_PRIVATE_KEY=$(grep INTEROP_PRIVATE_KEY ../.env | cut -d= -f2)
EOF

# Add token address if it exists in root .env
if grep -q "^VITE_TOKEN_ADDRESS=" ../.env; then
  grep "^VITE_TOKEN_ADDRESS=" ../.env >> .env
fi

# Install dependencies (first time only)
npm install

# Start wallet app
npm run dev
```

## Token Deployment

### Deploy with start-passkey-wallet-app.sh

```bash
./start-passkey-wallet-app.sh --deploy-token
```

### Deploy Separately

```bash
cd passkey-wallet-app

# Deploy to Chain A (default: http://localhost:3050)
./deploy-usd.sh

# Or specify custom RPC and private key
./deploy-usd.sh http://localhost:3050 0xYOUR_PRIVATE_KEY
```

The deployed token address will be automatically saved to both:

- `passkey-wallet-app/.env`
- Root `.env`

## Interop Testing

The interop features allow you to test cross-chain token transfers between two
local ZKsync chains.

> **📖 For detailed local chain setup, see
> [INTEROP_SETUP.md](INTEROP_SETUP.md)**

### Prerequisites

You need two local ZKsync chains running:

- Chain A: `http://localhost:3050`
- Chain B: `http://localhost:3051`

See [INTEROP_SETUP.md](INTEROP_SETUP.md) for complete setup instructions.

### Quick Setup (Summary)

#### Required Tools

You need Foundry/Anvil **v1.3.4** specifically:

```bash
# Install foundryup if you haven't already
curl -L https://foundry.paradigm.xyz | bash

# Install specific version
foundryup -v 1.3.4
```

#### Step 1: Clone ZKsync OS Server

```bash
git clone https://github.com/matter-labs/zksync-os-server.git
cd zksync-os-server
git checkout kl/interop-type-b
```

#### Step 2: Start Local Chains

Open **3 separate terminals** in the `zksync-os-server` directory:

##### Terminal 1: Start L1 (Anvil)

```bash
anvil --load-state ./local-chains/v31/zkos-l1-state.json --port 8545
```

##### Terminal 2: Start Chain A (Port 3050)

```bash
cargo run -- --config ./local-chains/v31/multiple-chains/chain1.json
```

##### Terminal 3: Start Chain B (Port 3051)

```bash
cargo run -- --config ./local-chains/v31/multiple-chains/chain2.json
```

#### Step 3: Start Interop Watcher (Optional)

The watcher synchronizes interop roots between chains. If you want to inspect
how it works:

```bash
# In a separate directory
git clone https://github.com/matter-labs/zksync-os-workflows.git
cd zksync-os-workflows
git checkout deniallugo-create-2-chains
# Or: git checkout kl/interop-type-b

./start-watcher.sh
```

**Note:** The UI already includes the same watcher code, so this step is
optional for basic testing.

### Setup

1. **Start local chains** (in separate terminals):

   ```bash
   # Terminal 1: Chain A on port 3050
   # Follow ZKsync local chain setup docs

   # Terminal 2: Chain B on port 3051
   # Follow ZKsync local chain setup docs
   ```

2. **Deploy token to Chain A**:

   ```bash
   ./start-passkey-wallet-app.sh --deploy-token
   # Or manually:
   cd passkey-wallet-app && ./deploy-usd.sh
   ```

3. **Start the application**:

   ```bash
   ./start-passkey-wallet-app.sh
   ```

4. **Use the Interop tab** in the wallet UI at <http://localhost:3000>

### Testing Workflow

1. Navigate to the **Interop** tab in the wallet
2. Click **Refresh Token Balances** to check connectivity
3. Use **Send Message** to test cross-chain messaging
4. Use **Transfer Tokens** to test cross-chain token transfers
5. Monitor the **Activity** tab to see L2-to-L1 finalization progress

> **📖 For detailed interop setup and troubleshooting, see
> [INTEROP_SETUP.md](INTEROP_SETUP.md)**

## Troubleshooting

### Services Won't Start

**Problem**: Error about missing environment variables

**Solution**:

1. Make sure you created `.env` from `.env.example`
2. Check that all required values are filled in (not placeholder values)
3. Verify your private key has the `0x` prefix

### Bundler Issues

**Problem**: Bundler fails to start or transactions fail

**Solution**:

1. Check that your private key has ETH on ZKsync OS testnet
2. Verify the RPC URL is correct: `https://zksync-os-testnet-alpha.zksync.dev/`
3. Check bundler logs: `tail -f logs/bundler.log`

### Relayer Issues

**Problem**: Transactions stuck in pending state

**Solution**:

1. Check that your private key has ETH on L1 Sepolia
2. Verify Alchemy API key is correct
3. Check relayer logs: `tail -f logs/relayer.log`
4. Note: L2-to-L1 finalization takes ~15 minutes on testnet

### Interop Features Not Working

**Problem**: Warning box or connection errors in Interop tab

**Solution**:

1. Verify both local chains are running on ports 3050 and 3051
2. Check that token is deployed: `grep VITE_TOKEN_ADDRESS .env`
3. Ensure `INTEROP_PRIVATE_KEY` has funds on both local chains

### Token Deployment Fails

**Problem**: deploy-usd.sh fails

**Solution**:

1. Install Foundry: `curl -L https://foundry.paradigm.xyz | bash && foundryup`
2. Install jq: `brew install jq` (macOS) or `sudo apt-get install jq` (Linux)
3. Check that local chain is running on port 3050
4. Verify the private key has funds on the target chain

### Port Conflicts

**Problem**: Service fails to start due to port already in use

**Solution**: Edit `.env` and change the conflicting port:

```bash
BUNDLER_PORT=4337        # Change to another port like 4338
RELAYER_STATUS_PORT=4340 # Change to another port like 4341
WALLET_APP_PORT=3000     # Change to another port like 3001
```

### View Service Status

Check if services are running:

```bash
# Check relayer status
curl http://localhost:4340/status

# Check bundler
curl http://localhost:4337/health

# Check wallet app
curl http://localhost:3000
```

### Clean Start

If you're experiencing issues, try a clean restart:

```bash
# 1. Stop all services (Ctrl+C in start-passkey-wallet-app.sh terminal)

# 2. Clean up processes
pkill -f "auto-finalize-daemon"
pkill -f "alto"
pkill -f "vite"

# 3. Remove old logs
rm -rf logs/

# 4. Restart
./start-passkey-wallet-app.sh
```

## Additional Resources

- **ZKsync OS Documentation**: <https://docs.zksync.io/>
- **Passkey/WebAuthn**: <https://webauthn.guide/>
- **ERC-4337**: <https://eips.ethereum.org/EIPS/eip-4337>

## Support

If you encounter issues not covered in this guide:

1. Check the service logs in `logs/` directory
2. Review the console output from `start-passkey-wallet-app.sh`
3. Open an issue on the GitHub repository with:

   - Description of the problem
   - Relevant log excerpts
   - Steps to reproduce
