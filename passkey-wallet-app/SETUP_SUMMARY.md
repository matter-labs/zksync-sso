# Passkey Wallet App - Setup Summary

## 🎯 What You Have

I've created a complete passkey wallet application with deployment
infrastructure for Ethereum Sepolia testnet.

## 📁 File Structure

```
ZKSync-SSO/
├── passkey-wallet-app/           # ✅ Your new wallet app
│   ├── index.html                # Beautiful UI with 3-step flow
│   ├── main.js                   # Passkey + WebAuthn integration
│   ├── package.json              # Dependencies (viem, @simplewebauthn)
│   ├── vite.config.js            # Build configuration
│   ├── README.md                 # Complete app documentation
│   └── SETUP_SUMMARY.md          # This file!
│
├── zksync-sso-contracts/         # ✅ Cloned contracts repository
│   ├── script/Deploy.s.sol       # Foundry deployment script
│   ├── deploy-sepolia.sh         # Custom Sepolia deployment script
│   ├── extract-addresses.js      # Address extraction utility
│   └── sepolia-addresses.json    # (Created after deployment)
│
└── DEPLOYMENT_GUIDE.md           # ✅ Step-by-step deployment guide
```

## 🚀 Next Steps (What You Need to Do)

### Step 1: Deploy Smart Contracts to Sepolia

You need to deploy the ZKsync SSO contracts to Sepolia. This is **required** for
the app to work.

#### 1.1 Get Sepolia ETH

- Visit <https://sepoliafaucet.com/>
- Request at least **0.5 Sepolia ETH** to your wallet
- This will cover deployment gas costs

#### 1.2 Run Deployment

```bash
cd zksync-sso/zksync-sso-contracts

# Set your private key
export PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE

# Deploy all contracts (takes ~2-3 minutes)
./deploy-sepolia.sh

# Extract the deployed addresses
node extract-addresses.js
```

This will deploy:

- ✅ EOAKeyValidator
- ✅ SessionKeyValidator
- ✅ WebAuthnValidator
- ✅ GuardianExecutor
- ✅ ModularSmartAccount (implementation)
- ✅ UpgradeableBeacon
- ✅ MSAFactory

### Step 2: Configure the Wallet App

After deployment, you'll have contract addresses. Update the app:

```bash
cd zksync-sso/passkey-wallet-app

# Open main.js and update these lines (around line 20):
```

Replace:

```javascript
const SEPOLIA_CONTRACTS = {
  webauthnValidator: "0x0000000000000000000000000000000000000000", // PLACEHOLDER
  factory: "0x0000000000000000000000000000000000000000", // PLACEHOLDER
  beacon: "0x0000000000000000000000000000000000000000", // PLACEHOLDER
};
```

With your deployed addresses from `sepolia-addresses.json`:

```javascript
const SEPOLIA_CONTRACTS = {
  webauthnValidator: "0xYOUR_WEBAUTHN_ADDRESS",
  factory: "0xYOUR_FACTORY_ADDRESS",
  beacon: "0xYOUR_BEACON_ADDRESS",
};
```

### Step 3: Set Up Bundler

You need an ERC-4337 bundler to submit transactions. Update `main.js`:

#### Option A: Pimlico (Recommended - Free tier available)

```javascript
const BUNDLER_URL = "https://api.pimlico.io/v2/sepolia/rpc?apikey=YOUR_API_KEY";
```

- Sign up: <https://dashboard.pimlico.io/>
- Create API key
- Copy to code

#### Option B: Alchemy (If you have Alchemy account)

```javascript
const BUNDLER_URL = "https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY";
```

#### Option C: Stackup (Public endpoint)

```javascript
const BUNDLER_URL = "https://api.stackup.sh/v1/node/sepolia";
```

### Step 4: Install and Run

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

App opens at `http://localhost:3000` 🎉

### Step 5: Test the Flow

1. **Create Passkey**

   - Enter your name
   - Click "Create Passkey"
   - Authenticate with Touch ID / Face ID / PIN

2. **Deploy Account**

   - Click "Deploy Account"
   - Wait ~15-30 seconds
   - Copy your smart account address

3. **Fund Account**

   - Go to <https://sepoliafaucet.com/>
   - Send Sepolia ETH to your smart account address
   - Click "Refresh Balance" in app

4. **Send Transaction**
   - Enter recipient address
   - Enter amount (e.g., 0.01 ETH)
   - Click "Send ETH"
   - Authenticate with passkey
   - Transaction submitted! 🚀

## 📋 Complete Checklist

### Pre-Deployment

- [ ] You have a wallet with 0.5+ Sepolia ETH
- [ ] Foundry is installed (already ✅)
- [ ] Contracts repository cloned (already ✅)

### Deployment

- [ ] Run `./deploy-sepolia.sh`
- [ ] Run `node extract-addresses.js`
- [ ] Copy `sepolia-addresses.json` addresses

### App Configuration

- [ ] Update contract addresses in `main.js`
- [ ] Configure bundler URL
- [ ] Run `npm install`
- [ ] Run `npm run dev`

### Testing

- [ ] Create a passkey
- [ ] Deploy smart account
- [ ] Fund account with Sepolia ETH
- [ ] Send test transaction
- [ ] Verify transaction on Sepolia Etherscan

## 🎨 What the App Does

### Current Features (✅ Working)

- **Passkey Creation**: Uses WebAuthn to create secure passkeys
- **Passkey Authentication**: Biometric/PIN authentication for transactions
- **Beautiful UI**: Clean 3-step flow with visual feedback
- **Network Connection**: Connected to Sepolia via Alchemy RPC

### After Deployment (🔧 Requires Your Setup)

- **Smart Account Deployment**: Deploy ERC-7579 modular accounts
- **Transaction Signing**: Sign UserOperations with passkeys
- **ETH Transfers**: Send transactions via ERC-4337 bundler

## 📖 Documentation

All documentation is ready:

1. **[DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md)** - Complete deployment
   instructions
2. **[passkey-wallet-app/README.md](./README.md)** - App documentation
3. **[This file]** - Quick summary

## 🔧 Architecture

```
Your Wallet App
    ↓
WebAuthn (Browser) → Creates/Uses Passkey
    ↓
Sepolia Contracts → WebAuthnValidator, MSAFactory
    ↓
ERC-4337 Bundler → Submits UserOperations
    ↓
Ethereum Sepolia → Executes Transactions
```

## ⚠️ Important Notes

1. **This is for Sepolia Testnet** - Not production mainnet
2. **You need to deploy contracts** - They're not deployed yet
3. **You need a bundler** - Free options available (Pimlico, Stackup)
4. **Passkeys are device-bound** - Use same device for auth

## 🆘 Need Help?

### Deployment Issues

- See: [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md)
- Check: Foundry installation, private key, Sepolia ETH balance

### App Issues

- See: [passkey-wallet-app/README.md](./README.md)
- Check: Browser compatibility, HTTPS/localhost, biometric setup

### Contract Questions

- Repo: <https://github.com/matter-labs/zksync-sso-contracts>
- Docs: <https://github.com/matter-labs/zksync-sso>

## 🎉 What's Next?

After successful deployment and testing:

1. ✅ You'll have a working passkey wallet on Sepolia
2. ✅ Users can create accounts without seed phrases
3. ✅ Transactions signed with biometric auth
4. 🚀 You can build on top of this foundation!

## Quick Command Reference

```bash
# Deploy contracts
cd zksync-sso-contracts
export PRIVATE_KEY=0x...
./deploy-sepolia.sh
node extract-addresses.js

# Run app
cd passkey-wallet-app
npm install
# (Update contract addresses in main.js first!)
npm run dev

# Build for production
npm run build
npm run preview
```

---

**Ready to deploy?** Start with Step 1 above! 🚀
