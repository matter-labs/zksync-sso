# ZKsync SSO Demo - Quick Start

Get up and running in 3 steps!

## Step 1: Install Prerequisites

```bash
# Install Foundry (for token deployment)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Install jq (for JSON parsing)
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq
```

## Step 2: Configure

```bash
# Copy configuration template
cp .env.example .env

# Edit .env and set these values:
# - DEPLOYER_PRIVATE_KEY: Your private key (must have ETH on Sepolia and ZKsync OS testnet)
# - L1_RPC_URL: Your Alchemy API URL for Sepolia
```

Example `.env`:

```bash
DEPLOYER_PRIVATE_KEY=0xYOUR_ACTUAL_PRIVATE_KEY_HERE
L1_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_ACTUAL_ALCHEMY_KEY_HERE
```

## Step 3: Start Everything

```bash
# Start all services (relayer, bundler, wallet app) + deploy token
./start-passkey-wallet-app.sh --deploy-token
```

**That's it!** Open <http://localhost:3000> in your browser.

---

## What Gets Started

| Service        | Port | URL                            |
| -------------- | ---- | ------------------------------ |
| Wallet App     | 3000 | <http://localhost:3000>        |
| Bundler        | 4337 | <http://localhost:4337>        |
| Relayer Status | 4340 | <http://localhost:4340/status> |

## Common Commands

```bash
# Start all services
./start-passkey-wallet-app.sh

# Start with token deployment
./start-passkey-wallet-app.sh --deploy-token

# Stop all services
# Press Ctrl+C in the terminal running start-passkey-wallet-app.sh

# View logs
tail -f logs/wallet.log
tail -f logs/bundler.log
tail -f logs/relayer.log

# Deploy token separately (to Chain A)
cd passkey-wallet-app && ./deploy-usd.sh
```

## Getting Test ETH

1. **Sepolia ETH**: <https://sepoliafaucet.com/>
2. **ZKsync OS Testnet ETH**: <https://portal.zksync.io/bridge/>

## Testing Cross-Chain (Interop) Features

The app includes an **Interop** tab for cross-chain token transfers. To use it:

1. **Set up local chains** - See [INTEROP_SETUP.md](INTEROP_SETUP.md) for
   detailed instructions
2. **Deploy token**: `./start-passkey-wallet-app.sh --deploy-token`
3. **Use Interop tab** in the wallet UI

**Note**: Interop features require local ZKsync chains running on ports 3050
& 3051.

## Need Help?

- [SETUP.md](SETUP.md) - Detailed setup, configuration, and troubleshooting
- [INTEROP_SETUP.md](INTEROP_SETUP.md) - Local chains setup for interop testing

## Directory Structure

```
.
├── .env                          # Your configuration (create from .env.example)
├── start-passkey-wallet-app.sh                  # Start everything with one command
├── QUICKSTART.md                 # This file
├── SETUP.md                      # Detailed setup guide
├── l2-to-l1-relayer/            # L2→L1 finalization daemon
├── packages/bundler/            # ERC-4337 bundler
└── passkey-wallet-app/          # Wallet frontend
    ├── deploy-usd.sh            # Deploy test token
    └── main.js                  # Wallet logic
```
