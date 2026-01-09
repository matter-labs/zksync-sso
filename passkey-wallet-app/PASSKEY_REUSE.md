# Passkey Reuse Feature

## Overview

The app now saves your passkey and account data to browser localStorage,
so you can reuse the same passkey and account across browser sessions without having to create a new one each time.

## How It Works

### Automatic Save

When you create a passkey or deploy an account, the data is automatically saved to your browser's localStorage:

```javascript
localStorage:
  - zksync_passkey_data: { credentialId, publicKey, userName }
  - zksync_account_address: "0x..."
```

### Automatic Load

When you open the app, it automatically checks for saved passkey data:

- ✅ If found: Restores your passkey and account state
- ❌ If not found: Shows the "Create Passkey" form

### What Gets Saved

1. **Passkey Data**:
   - Credential ID (base64url and hex formats)
   - Public key coordinates (x, y)
   - Username

2. **Account Data**:
   - Deployed account address (if deployed)

### What Doesn't Get Saved

- **Private Key**: NEVER leaves your device! The private key is stored securely in your device's authenticator
(Touch ID, Face ID, Windows Hello, etc.) and is never exposed to the app.

## Benefits

### No Need to Fund Multiple Accounts

- ✅ Create passkey once
- ✅ Fund the account once
- ✅ Reuse the same account every time you open the app

### Same Account, Same Funds

Since your account address is deterministic (based on the credential ID), you'll always get the same address:

- Same passkey → Same credential ID → Same account address
- Your funds stay in the same account

## Using the App

### First Time

1. Open the app
2. Create a passkey with your name
3. Deploy account (get your address)
4. Fund your account via faucet
5. Send transactions

### Subsequent Visits

1. Open the app
2. ✅ **Passkey and account automatically loaded!**
3. Your balance is displayed
4. Ready to send transactions immediately

## Resetting Your Passkey

If you want to start over with a new passkey:

1. Click the **"Reset Passkey"** button (shown after passkey creation)
2. Confirm the reset
3. Your stored data is cleared
4. You can now create a new passkey

**Note**: Resetting only clears the app's stored data. Your original passkey still exists in your browser's password manager.

## Browser Passkey Storage

### Where Are Passkeys Actually Stored?

The actual passkey (private key) is stored by your browser/OS:

- **Chrome**: Settings → Password Manager → Passkeys
- **Safari**: Settings → Passwords → [Website]
- **Windows**: Windows Hello
- **macOS**: iCloud Keychain

### Why Does the App Save Data?

The app saves:

- **Public key** (not secret, needed for transactions)
- **Credential ID** (not secret, identifies which passkey to use)
- **Account address** (not secret, your smart account address)

This allows the app to:

1. Know which passkey to request when signing
2. Calculate the correct account address
3. Show your account state immediately

## Security Notes

### ✅ Safe to Store

- Public key coordinates (x, y)
- Credential ID
- Account address
- Username

These are not secret values and cannot be used to access your account without the actual passkey authentication.

### 🔒 Never Stored

- Private key (stays in device authenticator)
- Passkey authentication results
- Transaction signatures (generated on-demand)

### Data Persistence

- Stored in browser localStorage
- Persists across browser sessions
- Cleared if you:
  - Click "Reset Passkey"
  - Clear browser data
  - Use incognito/private mode

## Multi-Device Support

### Same Browser, Same Device

✅ Passkey and account data persist

### Different Browser, Same Device

⚠️ Need to authenticate with the same passkey

- The passkey exists on your device
- But localStorage is per-browser
- Just authenticate when prompted, and the app will reconstruct the state

### Different Device

❌ Passkey doesn't transfer automatically

- Passkeys are device-bound by default
- You'd need to:
  1. Create a new passkey on the new device (will generate a different account)
  2. Or use a passkey provider that syncs across devices (e.g., iCloud Keychain, Google Password Manager)

## Technical Details

### LocalStorage Structure

```javascript
// Saved passkey data
{
  "credentialId": "base64url-encoded-id",
  "credentialIdHex": "0x...",
  "publicKey": {
    "x": "0x...",
    "y": "0x..."
  },
  "userName": "Your Name"
}
```

### Address Calculation

The account address is deterministically calculated from the credential ID:

```javascript
accountId = keccak256(credentialIdHex)
accountAddress = CREATE2(factory, accountId, initCodeHash)
```

This means:

- Same credential ID → Same account address
- Different credential ID → Different account address

### Signing Transactions

Even though public key data is stored, signing still requires:

1. WebAuthn authentication (biometric/PIN)
2. Device authenticator generates signature
3. Signature is used once and discarded

The app cannot sign transactions without your explicit authentication each time.

## FAQ

### Q: Is it safe to store passkey data in localStorage?

**A**: Yes! The data stored is public information (public key, credential ID).
The private key never leaves your device's secure authenticator.

### Q: What if I clear my browser data?

**A**: The app's stored data is cleared, but your passkey still exists in your browser's password manager.
Just authenticate again when prompted.

### Q: Can I use the same passkey on multiple devices?

**A**: Depends on your passkey provider:

- iCloud Keychain: Syncs across Apple devices
- Google Password Manager: Syncs across devices
- Platform authenticator: Device-specific

### Q: What if someone accesses my localStorage?

**A**: They would see your public key and account address, but they cannot:

- Sign transactions (need your biometric/device authentication)
- Access your private key (stored in secure enclave)
- Steal your funds (need passkey authentication for every transaction)

### Q: Do I need to fund a new account every time?

**A**: No! With passkey reuse, you use the same account and funds persist.

## Example Flow

### Session 1: Initial Setup

```
1. Open app → No saved data
2. Create passkey → Saved to localStorage
3. Deploy account → Saved to localStorage
4. Fund with 0.1 ETH
5. Send 0.01 ETH to friend
6. Balance: 0.09 ETH
7. Close browser
```

### Session 2: Return Later

```
1. Open app → Loads saved passkey ✅
2. Shows account address ✅
3. Shows balance: 0.09 ETH ✅
4. Ready to transact immediately ✅
```

No need to recreate passkey or refund account!

## Troubleshooting

### "Passkey not found" when signing

- The passkey was deleted from browser's password manager
- Click "Reset Passkey" and create a new one
- Note: New passkey = New account address

### "Stored data corrupted"

- App automatically clears corrupted data
- Create a new passkey

### Want to start fresh

- Click "Reset Passkey" button
- Or manually clear localStorage:

  ```javascript
  localStorage.removeItem('zksync_passkey_data');
  localStorage.removeItem('zksync_account_address');
  ```

## Summary

✅ **Passkey reuse is now fully implemented!**

- Automatically saves and loads passkey data
- Same account across sessions
- No need to fund multiple accounts
- Secure (private key never exposed)
- Easy reset if needed

Your passkey wallet is now production-ready for convenient daily use!
