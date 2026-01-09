# Account Deployment Script

## Overview

This script allows you to deploy a smart account using an EOA (Externally Owned Account) with Sepolia ETH,
instead of relying on the bundler to deploy it.

## Why Use This?

The bundler's counterfactual deployment can fail with `AA13 initCode failed or OOG` errors. Using this script, you can:

- Deploy the account from an EOA that you control
- Verify the deployment succeeded
- Then use the account normally in the app

## Setup

### 1. Create .env File

```bash
cp .env.example .env
```

Edit `.env` and add your private key:

```
DEPLOYER_PRIVATE_KEY=0x... # Your EOA private key (must have Sepolia ETH)
```

**⚠️ Security Warning**: Never commit `.env` to git! It's already in `.gitignore`.

### 2. Get Your Passkey Data

After creating a passkey in the app, check the browser console for:

- Credential ID (hex format)
- Public Key X
- Public Key Y

Example console output:

```
Credential ID: 0xd748b11b3a22f1d0615de02b03a9225005a94ee2dcb4aff6501f9ca381736f5c
Public Key X: 0x44adf115f2c6a670d535c8d2735e0c853688404ef8643b67e479fa3cd3531377
Public Key Y: 0xba0dbf4a6192e8f59de753a8218033b7bd957bf59c9408490bdcde780e641ace
```

## Usage

```bash
node deploy-account.js <credentialIdHex> <publicKeyX> <publicKeyY>
```

### Example

```bash
node deploy-account.js \
  0xd748b11b3a22f1d0615de02b03a9225005a94ee2dcb4aff6501f9ca381736f5c \
  0x44adf115f2c6a670d535c8d2735e0c853688404ef8643b67e479fa3cd3531377 \
  0xba0dbf4a6192e8f59de753a8218033b7bd957bf59c9408490bdcde780e641ace
```

## What It Does

1. **Validates** your deployer has Sepolia ETH
2. **Calculates** the account ID from credential ID
3. **Checks** if account is already deployed
4. **Deploys** the account via factory.deployAccount()
5. **Waits** for transaction confirmation
6. **Displays** the deployed account address

## Output

Successful deployment:

```
🚀 Deploying Smart Account...

Deployer Address: 0x627178c71c1e2391a2c40c711f3c5cf71b36dac9
Deployer Balance: 100000000000000000 wei

Account ID: 0x...
Expected Address: 0x...
Status: Not deployed yet

Deploying account...
Factory: 0xF33128d7Cd2ab37Af12B3a22D9dA79f928c2B450
Transaction Hash: 0x...
Waiting for confirmation...

✅ Account deployed successfully!
Block Number: 12345678
Gas Used: 500000
Account Address: 0x...

🎉 Done! Your account is now deployed and ready to use.
```

## After Deployment

Once deployed, you can:

1. **Refresh the app** - Your account will show as deployed
2. **Send transactions** - No more `AA20 account not deployed` errors
3. **Use the app normally** - All features work!

## Troubleshooting

### "DEPLOYER_PRIVATE_KEY not found"

- Make sure you created `.env` file
- Check the private key starts with `0x`

### "Deployer has no ETH"

- Fund your deployer address with Sepolia ETH
- Get from: <https://sepoliafaucet.com/>

### "Account already deployed"

- The account is already deployed! You can use it in the app
- No need to deploy again

### "Deployment failed"

- Check gas prices aren't too high
- Verify the contract addresses are correct
- Ensure your passkey data is correct (credential ID, public keys)

## Important Notes

1. **Origin Must Match**: The script uses `http://localhost:3000` as the origin.
  If you're using a different origin in the app, update line 90 in `deploy-account.js`.

2. **One-Time Operation**: You only need to deploy the account once. After that,
  it exists on-chain forever (for that specific passkey).

3. **Gas Costs**: Deployment costs approximately 0.005-0.01 ETH on Sepolia.

4. **Same Passkey = Same Account**: The account address is deterministic based on your credential ID.
  Same passkey always generates the same account address.

## Alternative: Deploy from App (Advanced)

If you want to avoid using this script, you can also deploy programmatically by funding the account address BEFORE deployment,
  then the first transaction will deploy it automatically.
  However, this is more complex and requires careful gas estimation.

## Security Best Practices

- ✅ Keep `.env` private (never commit)
- ✅ Use a dedicated deployer key (not your main wallet)
- ✅ Only deploy on testnet first
- ✅ Verify the account address matches what the app shows
- ❌ Never share your private key
- ❌ Don't reuse production keys for testing

## Helpful Commands

```bash
# Check deployer balance
cast balance 0x627178c71c1e2391a2c40c711f3c5cf71b36dac9 --rpc-url https://eth-sepolia.g.alchemy.com/v2/...

# Check if account is deployed
cast code <ACCOUNT_ADDRESS> --rpc-url https://eth-sepolia.g.alchemy.com/v2/...

# Get account balance
cast balance <ACCOUNT_ADDRESS> --rpc-url https://eth-sepolia.g.alchemy.com/v2/...
```

## Flow Diagram

```
┌─────────────────────┐
│  Create Passkey     │
│  in Browser App     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Copy Credential    │
│  & Public Key Data  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Run deploy-account │
│  node deploy-acc... │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Account Deployed!  │
│  ✅ On-chain        │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Fund Account       │
│  (via faucet)       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Use App Normally   │
│  Send Transactions  │
└─────────────────────┘
```

---

Happy deploying! 🚀
