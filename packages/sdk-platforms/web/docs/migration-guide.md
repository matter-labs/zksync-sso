# Migration Guide: Low-Level WASM API to Viem Integration

This guide helps you migrate from the low-level WASM API to the recommended viem-based session API.

## Why Migrate?

The viem integration provides:

- ✅ Better TypeScript types and developer experience
- ✅ Automatic gas estimation and nonce management
- ✅ Standard viem/account-abstraction patterns
- ✅ Built-in error handling and retries
- ✅ Session-based authentication out of the box

## Quick Comparison

### Before: Low-Level WASM API

```typescript
import {
  deploy_account,
  send_transaction_eoa,
  DeployAccountConfig,
  SendTransactionConfig,
} from 'zksync-sso-web-sdk/bundler';

// Deploy account
const config = new DeployAccountConfig(
  "http://localhost:8545",
  "0xBF0c79fD7b3eeF6Ac1bf67CEef3B2adED40ddC40",
  deployerPrivateKey,
  "0x3a09a0f6Cb773BCfdf606D21FAecFe5699f74718",
  null
);

const accountAddress = await deploy_account(
  "user-123",
  ["0x70997970C51812dc3A010C7d01b50e0d17dc79C8"],
  null,
  config
);

// Send transaction
const txConfig = new SendTransactionConfig(
  "http://localhost:8545",
  "http://localhost:4337",
  "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108"
);

const receipt = await send_transaction_eoa(
  txConfig,
  "0x3a09a0f6Cb773BCfdf606D21FAecFe5699f74718",
  signerPrivateKey,
  accountAddress,
  recipientAddress,
  "1000000000000000000",
  null
);
```

### After: Viem Integration

```typescript
import { createPublicClient, createBundlerClient, http, parseEther } from 'viem';
import { toSmartAccount } from 'viem/account-abstraction';
import { privateKeyToAccount } from 'viem/accounts';
import {
  getAccountInitCode,
  encodeExecuteCallData,
  generateEoaStubSignature,
  signEoaUserOperationHash,
} from 'zksync-sso-web-sdk/bundler';

// Setup clients
const publicClient = createPublicClient({
  chain,
  transport: http('http://localhost:8545'),
});

const owner = privateKeyToAccount(signerPrivateKey);

// Create smart account
const account = await toSmartAccount({
  client: publicClient,
  entryPoint: {
    address: '0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108',
    version: '0.8',
  },
  async getFactoryArgs() {
    return {
      factory: '0xBF0c79fD7b3eeF6Ac1bf67CEef3B2adED40ddC40',
      factoryData: await getAccountInitCode("user-123", [owner.address]),
    };
  },
  async encodeCallData(calls) {
    return await encodeExecuteCallData(calls, eoaValidatorAddress);
  },
  async getStubSignature() {
    return await generateEoaStubSignature(eoaValidatorAddress);
  },
  async signUserOperation({ userOperation }) {
    const hash = getUserOpHash(userOperation);
    return await signEoaUserOperationHash(hash, signerPrivateKey);
  },
});

// Create bundler client
const bundlerClient = createBundlerClient({
  account,
  transport: http('http://localhost:4337'),
  chain,
  client: publicClient,
});

// Send transaction
const userOpHash = await bundlerClient.sendUserOperation({
  calls: [
    {
      to: recipientAddress,
      value: parseEther('1'),
      data: '0x',
    },
  ],
});
```

## Step-by-Step Migration

### 1. Install Dependencies

```bash
pnpm add viem
```

### 2. Replace Configuration Classes

**Before**:

```typescript
const deployConfig = new DeployAccountConfig(
  rpcUrl,
  factoryAddress,
  deployerKey,
  eoaValidator,
  webauthnValidator
);

const txConfig = new SendTransactionConfig(
  rpcUrl,
  bundlerUrl,
  entryPointAddress
);
```

**After**:

```typescript
const publicClient = createPublicClient({
  chain,
  transport: http(rpcUrl),
});

// Configuration is passed directly to functions
```

### 3. Migrate Account Deployment

**Before**:

```typescript
const accountAddress = await deploy_account(
  userId,
  [signerAddress],
  null,
  deployConfig
);
```

**After**:

```typescript
const account = await toSmartAccount({
  client: publicClient,
  entryPoint: {
    address: entryPointAddress,
    version: '0.8',
  },
  async getFactoryArgs() {
    return {
      factory: factoryAddress,
      factoryData: await getAccountInitCode(userId, [signerAddress]),
    };
  },
  // ... other methods
});

const accountAddress = await account.getAddress();
```

### 4. Migrate Transaction Sending

**Before**:

```typescript
const receipt = await send_transaction_eoa(
  txConfig,
  eoaValidatorAddress,
  privateKey,
  accountAddress,
  recipientAddress,
  valueInWei,
  callData
);
```

**After**:

```typescript
const bundlerClient = createBundlerClient({
  account,
  transport: http(bundlerUrl),
  chain,
  client: publicClient,
});

const userOpHash = await bundlerClient.sendUserOperation({
  calls: [
    {
      to: recipientAddress,
      value: BigInt(valueInWei),
      data: callData || '0x',
    },
  ],
});

// Wait for receipt if needed
const receipt = await publicClient.waitForTransactionReceipt({
  hash: userOpHash,
});
```

### 5. Migrate Value Handling

**Before**:

```typescript
const value = "1000000000000000000"; // String in wei
```

**After**:

```typescript
import { parseEther } from 'viem';

const value = parseEther('1'); // bigint
```

### 6. Migrate Passkey Integration

**Before**:

```typescript
const passkeyPayload = new PasskeyPayload(
  publicKeyX,
  publicKeyY,
  credentialId
);

const accountAddress = await deploy_account(
  userId,
  [],
  passkeyPayload,
  deployConfig
);
```

**After**:

```typescript
// Use viem's WebAuthn support (coming soon in SDK)
// For now, use low-level API for passkeys
```

## Session Support

The viem integration includes built-in session support, which wasn't available in the low-level API:

```typescript
import { createSessionAction, createSessionSmartAccount } from 'zksync-sso-web-sdk';

// Create session
const sessionSpec = {
  signer: sessionSignerAddress,
  expiresAt: BigInt(Date.now() / 1000 + 3600),
  feeLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther('1'),
    period: 0n,
  },
  callPolicies: [],
  transferPolicies: [/* ... */],
};

await createSessionAction(bundlerClient, {
  sessionSpec,
  contracts: { sessionValidator: sessionValidatorAddress },
});

// Use session
const sessionAccount = await createSessionSmartAccount(publicClient, {
  entryPoint: entryPointAddress,
  factory: factoryAddress,
  accountAddress,
  signerPrivateKey: sessionPrivateKey,
  sessionValidatorAddress,
  sessionSpec,
});
```

## Error Handling

### Before: Manual Error Checking

```typescript
try {
  const receipt = await send_transaction_eoa(/* ... */);
  console.log('Success:', receipt);
} catch (error) {
  // Generic error object
  console.error('Error:', error);
}
```

### After: Structured Error Handling

```typescript
import { 
  UserOperationExecutionError, 
  EstimateGasExecutionError 
} from 'viem/account-abstraction';

try {
  const userOpHash = await bundlerClient.sendUserOperation({
    calls: [/* ... */],
  });
} catch (error) {
  if (error instanceof UserOperationExecutionError) {
    console.error('UserOp failed:', error.cause);
    // Access specific error details
    console.log('Revert reason:', error.details);
  } else if (error instanceof EstimateGasExecutionError) {
    console.error('Gas estimation failed:', error.message);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## Type Safety Improvements

### Before: Loose Types

```typescript
const accountAddress: string = await deploy_account(/* ... */);
const value: string = "1000000000000000000";
```

### After: Strict Types

```typescript
import type { Address, Hex } from 'viem';

const accountAddress: Address = await account.getAddress();
const value: bigint = parseEther('1');
const callData: Hex = '0x1234...';
```

## Testing

### Before: Manual Setup

```typescript
// Manual anvil account setup
const DEPLOYER_KEY = "0xac09...";
const SIGNER_KEY = "0x59c6...";
```

### After: Viem Test Utilities

```typescript
import { createTestClient, http } from 'viem';
import { foundry } from 'viem/chains';

const testClient = createTestClient({
  chain: foundry,
  transport: http(),
  mode: 'anvil',
});

// Get test accounts
const accounts = await testClient.getAddresses();

// Mine blocks
await testClient.mine({ blocks: 1 });

// Set balance
await testClient.setBalance({
  address: accounts[0],
  value: parseEther('1000'),
});
```

## Performance Considerations

Both APIs use the same underlying WASM module, so performance is identical. However, the viem integration provides:

- **Better caching**: Automatic caching of account addresses and nonces
- **Batch operations**: Easier to batch multiple operations
- **Gas optimization**: Automatic gas estimation with retries

## When to Use Low-Level API

The low-level WASM API is still useful for:

- **Custom integrations**: Non-viem environments
- **Minimal bundle size**: Avoid viem dependency
- **Direct control**: Need precise control over all parameters
- **Passkey-only flows**: Until viem integration supports passkeys

## Migration Checklist

- [ ] Install viem dependency
- [ ] Replace `DeployAccountConfig` and `SendTransactionConfig` with viem clients
- [ ] Update `deploy_account` to `toSmartAccount`
- [ ] Update `send_transaction_eoa` to `bundlerClient.sendUserOperation`
- [ ] Convert string values to `bigint` using `parseEther`
- [ ] Update error handling to use viem error types
- [ ] Test with local development environment
- [ ] Update TypeScript types (`string` → `Address`, `Hex`)
- [ ] Consider adding session support

## Need Help?

- See [Session API Guide](./session-api.md) for session usage
- See [examples/demo-app](../../../examples/demo-app) for working examples
- Check [viem documentation](https://viem.sh) for viem-specific questions

## Feedback

If you encounter issues during migration or have suggestions for improving the viem integration, please open an issue in the repository.
