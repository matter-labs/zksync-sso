# ZKsync SSO Passkey Wallet (SDK Version)

This is a rewritten version of the passkey wallet app using the official **zksync-sso SDK**.

## Features

- ✅ Create passkeys using `registerNewPasskey` from SDK
- ✅ Deploy accounts using `deployModularAccount` from SDK
- ✅ Sign transactions using `createZksyncPasskeyClient` from SDK
- ✅ Transfer ETH with passkey authentication
- ✅ Persistent storage (passkey reuse)

## Quick Start

```bash
npm install
npm run dev
```

Open <http://localhost:3000>

## How It Works

### 1. Create Passkey

Uses `registerNewPasskey` from `zksync-sso/client/passkey`:

```js
const result = await registerNewPasskey({
  userName: 'alice',
  userDisplayName: 'Alice',
});
```

Returns:

- `credentialId`: Unique passkey identifier
- `credentialPublicKey`: P-256 public key bytes

### 2. Deploy Account

Uses `deployModularAccount` from `zksync-sso/client`:

```js
const result = await deployModularAccount(deployerClient, {
  accountFactory: CONTRACTS.accountFactory,
  installNoDataModules: [],
  owners: [], // No EOA owners, passkey only
  passkeyModule: {
    location: CONTRACTS.passkey,
    credentialId,
    credentialPublicKey,
  },
  sessionModule: {
    location: CONTRACTS.session,
  },
});
```

Deploys a modular smart account with:

- WebAuthn validator module (passkey auth)
- Session key validator module (for future sessions)

### 3. Transfer ETH

Uses `createZksyncPasskeyClient` to create a viem-compatible wallet client:

```js
const passkeyClient = createZksyncPasskeyClient({
  address: accountAddress,
  credentialPublicKey,
  userName,
  userDisplayName,
  contracts: CONTRACTS,
  chain: sepolia,
  transport: http(RPC_URL),
});

// Use like a normal viem wallet client
const hash = await passkeyClient.sendTransaction({
  to: recipient,
  value: parseEther(amount),
});
```

When signing, the user is prompted for passkey authentication (Touch ID, Face ID, etc).

## Configuration

### Contract Addresses (Sepolia)

```js
const CONTRACTS = {
  passkey: "0xAbcB5AB6eBb69F4F5F8cf1a493F56Ad3d28562bd",
  session: "0x09fbd5b956AF5c64C7eB4fb473E7E64DAF0f79D7",
  accountFactory: "0xF33128d7Cd2ab37Af12B3a22D9dA79f928c2B450",
  oidcKeyRegistry: "0x0000000000000000000000000000000000000000",
};
```

### Deployer Private Key

The app uses a hardcoded private key for deploying accounts. In production, you should:

- Remove the private key from the code
- Use environment variables
- Use a dedicated deployment service
- Or let users fund their accounts and deploy via bundler

Current deployer:

```js
const DEPLOYER_PRIVATE_KEY = '0xef506537558847aa991149381c4fedee8fe1252cf868986ac1692336530ec85c';
```

## SDK Benefits

Compared to the previous manual implementation:

1. **No manual WebAuthn handling**: SDK handles all WebAuthn ceremony
2. **No manual signature encoding**: SDK formats signatures correctly
3. **No bundler API calls**: SDK uses native viem actions
4. **Type safety**: Full TypeScript support with proper types
5. **Better error handling**: SDK provides clear error messages
6. **Future-proof**: SDK updates automatically support new features

## Architecture

```
┌─────────────────────────────┐
│  Browser App (Vite + Vanilla JS)
│  passkey-wallet-app         │
└─────────┬───────────────────┘
          │
          ├─► zksync-sso/client/passkey
          │   └─► registerNewPasskey()
          │       requestPasskeyAuthentication()
          │
          ├─► zksync-sso/client
          │   └─► deployModularAccount()
          │       createZksyncPasskeyClient()
          │
          └─► viem
              ├─► createPublicClient() (read balance)
              ├─► createWalletClient() (deploy account)
              └─► passkey client actions (send tx)
```

## SDK Dependencies

```json
{
  "dependencies": {
    "@simplewebauthn/browser": "^13.2.2",
    "@simplewebauthn/server": "^13.2.2",
    "viem": "^2.30.0",
    "zksync-sso": "^0.4.3"
  }
}
```

## Differences from Previous Version

| Feature | Old Version | SDK Version |
|---------|-------------|-------------|
| Passkey creation | Manual `startRegistration` | `registerNewPasskey()` |
| Public key extraction | Manual COSE parsing | Handled by SDK |
| Account deployment | Manual factory call | `deployModularAccount()` |
| Signature format | Manual encoding | Handled by SDK |
| Transaction signing | Manual UserOp creation | `passkeyClient.sendTransaction()` |
| Bundler calls | Manual fetch to bundler | Handled by SDK |

## Testing

1. Create a passkey (uses your device biometrics)
2. Deploy account (uses provided deployer key)
3. Fund account from faucet: <https://sepoliafaucet.com/>
4. Transfer ETH to another address
5. Verify transaction on Etherscan

## Troubleshooting

### "Can't identify expectedOrigin"

- Make sure you're running on localhost:3000
- Or manually provide `expectedOrigin: 'http://localhost:3000'`

### "Deployment failed"

- Check deployer account has ETH
- Verify contract addresses are correct
- Check Sepolia RPC is responding

### "Passkey authentication failed"

- Ensure biometric auth is enabled on your device
- Try using a different browser
- Check browser console for detailed errors

## Next Steps

- Add session key support for gasless transactions
- Implement account recovery
- Add ERC-20 token transfers
- Integrate paymaster for sponsored transactions
- Deploy to production (Mainnet)

## Resources

- [ZKsync SSO Docs](https://docs.zksync.io/build/zksync-sso)
- [SDK Repository](https://github.com/matter-labs/zksync-sso)
- [SDK Client Reference](../../docs/sdk/client/README.md)
