# Passkey Wallet App

A simple demonstration app for creating a passkey-based Ethereum wallet on Sepolia testnet using ZKsync SSO contracts.

## Features

1. **Create Passkey** - Generate a WebAuthn passkey credential stored on your device
2. **Deploy Smart Account** - Deploy an ERC-4337 modular smart account
3. **Transfer ETH** - Send ETH using passkey authentication via bundler
4. **Passkey Reuse** - Automatically saves and reloads your passkey data across sessions (no need to fund multiple accounts!)

## Quick Start

### Prerequisites

- Node.js 18+ installed
- A modern browser with WebAuthn support (Chrome, Edge, Safari, Firefox)
- HTTPS or localhost (required for WebAuthn)
- **ZKsync SSO contracts deployed to Sepolia** (see deployment guide below)

### Installation

```bash
# Install dependencies
npm install
```

### Running the App

```bash
# Start development server
npm run dev

# The app will open at http://localhost:3000
```

## 🚀 Complete Setup Guide

To make this app fully functional, you need to deploy the ZKsync SSO contracts to Sepolia.

### Step 1: Deploy Contracts to Sepolia

Follow the comprehensive deployment guide:

📖 **[See DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md)**

Quick summary:

```bash
# Navigate to contracts repo
cd ../zksync-sso-contracts

# Set your private key (wallet with 0.5+ Sepolia ETH)
export PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE

# Deploy contracts
./deploy-sepolia.sh

# Extract deployed addresses
node extract-addresses.js
```

### Step 2: Configure the App

After deploying contracts, update `main.js` with your deployed addresses:

1. Open `main.js`
2. Find the `SEPOLIA_CONTRACTS` constant (around line 20)
3. Replace placeholder addresses with your deployed contract addresses:

```javascript
const SEPOLIA_CONTRACTS = {
  webauthnValidator: "0xYOUR_WEBAUTHN_VALIDATOR_ADDRESS",
  factory: "0xYOUR_FACTORY_ADDRESS",
  beacon: "0xYOUR_BEACON_ADDRESS",
};
```

### Step 3: Set Up Bundler

Update the bundler URL in `main.js`:

#### Option A: Pimlico (Recommended)

```javascript
const BUNDLER_URL = 'https://api.pimlico.io/v2/sepolia/rpc?apikey=YOUR_API_KEY';
```

Get API key: <https://dashboard.pimlico.io/>

#### Option B: Alchemy

```javascript
const BUNDLER_URL = 'https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY';
```

#### Option C: Stackup

```javascript
const BUNDLER_URL = 'https://api.stackup.sh/v1/node/sepolia';
```

### Step 4: Run and Test

```bash
npm run dev
```

Then:

1. Open <http://localhost:3000>
2. Create a passkey
3. Deploy your smart account
4. Fund it with Sepolia ETH from <https://sepoliafaucet.com/>
5. Send a test transaction!

## Passkey Reuse

**New Feature**: Your passkey and account data are automatically saved to browser localStorage!

### Benefits

- ✅ **No need to create a new passkey every time** - Your passkey data persists across sessions
- ✅ **Same account, same funds** - Reuse the same smart account address
- ✅ **Instant ready** - Open the app and your wallet is immediately available

### How It Works

1. **First Time**: Create passkey → Deploy account → Fund account
2. **Next Time**: Open app → **Automatically loaded** → Ready to transact!

### Reset Passkey

If you want to start fresh, click the **"Reset Passkey"** button to clear your data and create a new passkey.

📖 **[Read more about Passkey Reuse →](./PASSKEY_REUSE.md)**

## How It Works

### 1. Passkey Creation

Uses WebAuthn API to create a secure passkey:

- Stored on your device (Touch ID, Face ID, Windows Hello)
- No seed phrases or private keys to manage
- Public key identifies your account

### 2. Account Deployment

Deploys an ERC-7579 modular smart account:

- Uses passkey for authentication
- Counterfactual deployment via factory
- WebAuthn validator module installed

### 3. ETH Transfer

Sends transactions via ERC-4337:

- Creates UserOperation
- Signs with passkey authentication
- Submitted through bundler
- Executes on-chain

## Architecture

```
┌─────────────────┐
│  Passkey Wallet │
│      App        │
└────────┬────────┘
         │
         ├─► WebAuthn API (Browser)
         │   └─► Passkey Authentication
         │
         ├─► ZKsync SSO Contracts (Sepolia)
         │   ├─► WebAuthnValidator
         │   ├─► MSAFactory
         │   └─► ModularSmartAccount
         │
         └─► ERC-4337 Bundler
             └─► UserOperation Execution
```

## Current Implementation Status

### ✅ Implemented

- Passkey creation via WebAuthn
- Passkey authentication
- Basic UI flow
- Network configuration

### ⚠️ Requires Deployment

To make fully functional, you need to:

1. ✅ Deploy ZKsync SSO contracts (see [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md))
2. ✅ Update contract addresses in code
3. ✅ Configure bundler endpoint

### 🔧 To Implement (After Deployment)

Replace the simulated functions in `main.js`:

```javascript
// Replace simulated deployment with:
import { prepareDeploySmartAccount } from 'zksync-sso-4337/client';

// Replace simulated transfer with:
import { createPasskeyClient } from 'zksync-sso-4337/client/passkey';
```

## Network Configuration

- **Network**: Ethereum Sepolia Testnet
- **Chain ID**: 11155111
- **RPC URL**: <https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU>
- **Bundler**: Configure based on provider

## Browser Compatibility

- Chrome/Edge 90+
- Safari 14+
- Firefox 93+

**Requirements**:

- HTTPS or localhost
- Biometric authentication enabled

## Troubleshooting

### "WebAuthn not supported"

- Use modern browser
- Ensure HTTPS or localhost

### "No authenticator available"

- Enable biometric auth on device
- Windows: Set up Windows Hello
- Mac: Enable Touch ID

### "Deployment failed"

- Check contract addresses are correct
- Verify bundler URL
- Check browser console for errors

### "Transaction failed"

- Ensure account has Sepolia ETH
- Verify passkey authentication completed
- Check bundler is responding

## Deployment Checklist

Before using the app:

- [ ] Contracts deployed to Sepolia
- [ ] Contract addresses updated in `main.js`
- [ ] Bundler URL configured
- [ ] Dependencies installed (`npm install`)
- [ ] App running (`npm run dev`)
- [ ] Smart account created and funded
- [ ] Test transaction sent successfully

## Security Notes

⚠️ **Testnet Only**: This is a testnet demonstration app. For production:

- Audit all smart contracts
- Use production bundler infrastructure
- Implement proper error handling
- Add transaction confirmations
- Set spending limits

## Resources

- [ZKsync SSO](https://github.com/matter-labs/zksync-sso)
- [ZKsync SSO Contracts](https://github.com/matter-labs/zksync-sso-contracts)
- [ERC-4337](https://eips.ethereum.org/EIPS/eip-4337)
- [ERC-7579](https://erc7579.com/)
- [WebAuthn Guide](https://webauthn.guide/)

## Support

- Deployment issues? See [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md)
- Contract questions? Check [zksync-sso-contracts](https://github.com/matter-labs/zksync-sso-contracts)
- General issues? [ZKsync SSO Issues](https://github.com/matter-labs/zksync-sso/issues)
