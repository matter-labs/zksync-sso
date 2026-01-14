# Interop Testing - Local Chain Setup

This guide explains how to set up local ZKsync chains for testing cross-chain
(interop) functionality.

## Overview

For interop testing, you need:

- **L1 Chain**: Anvil (port 8545)
- **Chain A**: ZKsync OS (port 3050)
- **Chain B**: ZKsync OS (port 3051)
- **Interop Watcher** (optional): Synchronizes interop roots between chains

## Prerequisites

### Required Tools

- **Rust & Cargo**: For running ZKsync OS server

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Foundry/Anvil v1.3.4** (specific version required):

  ```bash
  # Install foundryup
  curl -L https://foundry.paradigm.xyz | bash

  # Install v1.3.4
  foundryup -v 1.3.4

  # Verify version
  anvil --version  # Should show: anvil 0.2.0 (1.3.4 ...)
  ```

## Setup Instructions

### Step 1: Clone ZKsync OS Server

```bash
git clone https://github.com/matter-labs/zksync-os-server.git
cd zksync-os-server
git checkout kl/interop-type-b
```

### Step 2: Start Local Chains

You need **3 separate terminal windows**, all in the `zksync-os-server`
directory:

#### Terminal 1: Start L1 (Anvil)

```bash
cd zksync-os-server
anvil --load-state ./local-chains/v31/zkos-l1-state.json --port 8545
```

**What to expect:**

- Anvil starts on `http://localhost:8545`
- Pre-funded accounts available
- L1 contracts pre-deployed

#### Terminal 2: Start Chain A

```bash
cd zksync-os-server
cargo run -- --config ./local-chains/v31/multiple-chains/chain1.json
```

**What to expect:**

- Chain starts on `http://localhost:3050`
- Connects to L1 on port 8545
- First startup may take a few minutes to compile

#### Terminal 3: Start Chain B

```bash
cd zksync-os-server
cargo run -- --config ./local-chains/v31/multiple-chains/chain2.json
```

**What to expect:**

- Chain starts on `http://localhost:3051`
- Connects to L1 on port 8545
- Interop-enabled to communicate with Chain A

### Step 3: Start Interop Watcher (Optional)

The interop watcher synchronizes interop roots between chains, enabling
cross-chain message verification.

**Note:** The passkey wallet app already includes watcher functionality, so this
step is optional for basic testing. Use it if you want to inspect how the
synchronization works.

```bash
# In a separate directory (not in zksync-os-server)
git clone https://github.com/matter-labs/zksync-os-workflows.git
cd zksync-os-workflows

# Checkout branch (either works)
git checkout deniallugo-create-2-chains
# OR
git checkout kl/interop-type-b

# Start the watcher
./start-watcher.sh
```

**What the watcher does:**

- Monitors Chain A and Chain B for new batches
- Fetches interop roots from each chain
- Submits roots to the opposite chain
- Enables cross-chain message finalization

## Verification

Check that all services are running:

```bash
# Check L1 (Anvil)
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check Chain A
curl -X POST http://localhost:3050 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check Chain B
curl -X POST http://localhost:3051 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

All three should return valid JSON-RPC responses.

## Testing Interop with the Wallet App

Once local chains are running:

### 1. Deploy Test Token

```bash
cd passkey-wallet-app
./deploy-usd.sh \
  http://localhost:3050 \
  0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110
```

The token address will be automatically saved to `.env` files.

### 2. Start the Wallet App

```bash
# From project root
./start-passkey-wallet-app.sh
```

### 3. Test Interop Features

1. Open <http://localhost:3000>
2. Create passkey and deploy account (if not done already)
3. Navigate to **Interop** tab
4. Click **Refresh Token Balances** to verify connectivity
5. Use **Send Message** to test cross-chain messaging
6. Use **Transfer Tokens** to test cross-chain token transfers

## Default Accounts

The local chains come with pre-funded accounts:

```
Private Key: 0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110
Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Balance: ~10000 ETH on both Chain A and Chain B
```

This is Anvil's default rich account #1.

## Troubleshooting

### "Connection refused" errors

- Ensure all 3 chains are running (L1, Chain A, Chain B)
- Check that ports 8545, 3050, 3051 are not in use by other applications
- Wait a few seconds after starting chains before testing

### "Token not deployed" warning

- Run the token deployment script first (see step 1 above)
- Make sure you're deploying to Chain A (port 3050)

### Interop transfers timing out

- Make sure the watcher is running (or the wallet app is handling root
  synchronization)
- Wait for batches to be created on both chains (~10-30 seconds)
- Check that both chains are producing blocks

### Anvil version mismatch

If you see errors about incompatible state files:

```bash
# Check your version
anvil --version

# Reinstall v1.3.4 if needed
foundryup -v 1.3.4
```

## Clean Restart

If you need to start fresh:

```bash
# Stop all services (Ctrl+C in each terminal)

# Clean up processes
pkill -f anvil
pkill -f "cargo run"

# Restart following steps 2-3 above
```

## Next Steps

- See [SETUP.md](SETUP.md) for full application setup
- See [QUICKSTART.md](QUICKSTART.md) for quick start guide
- See
  [passkey-wallet-app/INTEROP_TESTING.md](passkey-wallet-app/INTEROP_TESTING.md)
  for detailed interop testing guide
