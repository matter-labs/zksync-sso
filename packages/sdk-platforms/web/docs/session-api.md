# Session API Guide

This guide explains how to use the session-based transaction API with the zkSync SSO viem integration.

## Overview

Sessions allow you to delegate transaction signing authority to a specific address (the "session signer") within defined security constraints:

- **Time limits**: Sessions expire after a specified timestamp
- **Fee limits**: Control maximum gas fees consumed
- **Transfer policies**: Restrict ETH transfers to specific addresses and amounts
- **Call policies**: Control which smart contracts can be called

This enables use cases like:

- Temporary delegation to a backend service
- Automated recurring payments
- Gas-efficient batch operations
- Mobile app experiences without constant wallet prompts

## Core Concepts

### Session Specification

A `SessionSpec` defines all the rules for a session:

```typescript
interface SessionSpec {
  signer: Address;           // Address that can sign with this session
  expiresAt: bigint;         // Unix timestamp (seconds) when session expires
  feeLimit: FeeLimit;        // Gas fee restrictions
  callPolicies: CallPolicy[];        // Smart contract interaction rules
  transferPolicies: TransferPolicy[]; // ETH transfer rules
}
```

### Fee Limits

Three types of fee limiting:

```typescript
enum LimitType {
  Unlimited = 0,  // No limit (use with caution!)
  Lifetime = 1,   // Total limit over session lifetime
  Allowance = 2,  // Limit resets every `period` seconds
}

interface FeeLimit {
  limitType: LimitType;
  limit: bigint;     // Maximum fees in wei
  period: bigint;    // Period duration (only for Allowance)
}
```

**Examples**:

```typescript
// No limit (development only!)
const unlimitedFees = {
  limitType: LimitType.Unlimited,
  limit: 0n,
  period: 0n,
};

// 1 ETH total for session lifetime
const lifetimeFees = {
  limitType: LimitType.Lifetime,
  limit: parseEther('1'),
  period: 0n,
};

// 0.1 ETH per hour (resets every hour)
const hourlyFees = {
  limitType: LimitType.Allowance,
  limit: parseEther('0.1'),
  period: 3600n,  // 1 hour in seconds
};
```

### Transfer Policies

Control ETH transfers:

```typescript
interface TransferPolicy {
  target: Address;           // Allowed recipient address
  maxValuePerUse: bigint;    // Max ETH per transaction
  valueLimit: ValueLimit;    // Total/periodic value limits
}

interface ValueLimit {
  limitType: LimitType;
  limit: bigint;
  period: bigint;
}
```

**Example**:

```typescript
// Allow sending up to 0.01 ETH per transaction to a specific address
// Total lifetime limit: 1 ETH
const transferPolicy = {
  target: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb' as Address,
  maxValuePerUse: parseEther('0.01'),
  valueLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther('1'),
    period: 0n,
  },
};
```

### Call Policies

Control smart contract interactions:

```typescript
interface CallPolicy {
  target: Address;        // Contract address
  selector: `0x${string}`; // Function selector (4 bytes)
  maxValuePerUse: bigint; // Max ETH per call
  valueLimit: ValueLimit; // Total/periodic value limits
  constraints: ParamCondition[]; // Function parameter restrictions
}
```

**Example**:

```typescript
// Allow calling USDC.transfer() with amount <= 100 USDC
const callPolicy = {
  target: USDC_ADDRESS,
  selector: '0xa9059cbb', // transfer(address,uint256)
  maxValuePerUse: 0n,      // No ETH value
  valueLimit: {
    limitType: LimitType.Unlimited,
    limit: 0n,
    period: 0n,
  },
  constraints: [
    {
      index: 1,  // Second parameter (amount)
      condition: ParamConditionType.LessThanOrEqual,
      refValue: '100000000', // 100 USDC (6 decimals)
    },
  ],
};
```

## Creating a Session

### Step 1: Define Session Spec

```typescript
import { parseEther } from 'viem';
import { LimitType } from '@zksync-sso/viem-session-sdk/bundler';

const sessionSpec = {
  signer: sessionSignerAddress, // Address that will use this session
  expiresAt: BigInt(Math.floor(Date.now() / 1000) + 86400), // 24 hours
  feeLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther('1'), // 1 ETH max fees
    period: 0n,
  },
  callPolicies: [],
  transferPolicies: [
    {
      target: allowedRecipient,
      maxValuePerUse: parseEther('0.001'), // 0.001 ETH per tx
      valueLimit: {
        limitType: LimitType.Unlimited,
        limit: 0n,
        period: 0n,
      },
    },
  ],
};
```

### Step 2: Create Session On-Chain

```typescript
import { createBundlerClient, http } from 'viem/account-abstraction';
import { createSessionAction } from '@zksync-sso/viem-session-sdk';

// Create bundler client with EOA-controlled account
const bundlerClient = createBundlerClient({
  account: eoaAccount, // Your main smart account
  transport: http(BUNDLER_URL),
  chain,
  client: publicClient,
});

// Create the session
const result = await createSessionAction(bundlerClient, {
  sessionSpec,
  contracts: {
    sessionValidator: SESSION_VALIDATOR_ADDRESS,
  },
});

console.log('Session created:', result.userOpHash);
```

### Step 3: Wait for Confirmation

```typescript
// Wait for the UserOperation to be included
const receipt = await publicClient.waitForTransactionReceipt({
  hash: result.userOpHash,
});

console.log('Session active at block:', receipt.blockNumber);
```

## Using a Session

### Step 1: Create Session Account

```typescript
import { createSessionSmartAccount } from '@zksync-sso/viem-session-sdk';

const sessionAccount = await createSessionSmartAccount(publicClient, {
  entryPoint: ENTRY_POINT_ADDRESS,
  factory: FACTORY_ADDRESS,
  accountAddress: ACCOUNT_ADDRESS, // The smart account that owns the session
  signerPrivateKey: SESSION_PRIVATE_KEY, // Session signer's private key
  sessionValidatorAddress: SESSION_VALIDATOR_ADDRESS,
  sessionSpec, // MUST match the spec used in createSessionAction
});
```

**Critical**: The `sessionSpec` here **must be identical** to the spec used when creating the session. Any difference (even `0x000...000` vs an actual address) will cause `AA23 SessionNotActive` errors.

### Step 2: Send Transactions

```typescript
const sessionBundlerClient = createBundlerClient({
  account: sessionAccount,
  transport: http(BUNDLER_URL),
  chain,
  client: publicClient,
});

// Send a transaction
const userOpHash = await sessionBundlerClient.sendUserOperation({
  calls: [
    {
      to: allowedRecipient, // Must match transfer policy
      value: parseEther('0.001'), // Must be <= maxValuePerUse
      data: '0x',
    },
  ],
});

console.log('Transaction sent:', userOpHash);
```

## Complete Example

```typescript
import { createPublicClient, createBundlerClient, http, parseEther } from 'viem';
import { zkSyncSepoliaTestnet } from 'viem/chains';
import {
  createSmartAccount,
  createSessionSmartAccount,
  createSessionAction,
  LimitType,
} from '@zksync-sso/viem-session-sdk';

// Setup
const publicClient = createPublicClient({
  chain: zkSyncSepoliaTestnet,
  transport: http(),
});

// 1. Create main smart account (one-time setup)
const mainAccount = await createSmartAccount(publicClient, {
  entryPoint: ENTRY_POINT_ADDRESS,
  factory: FACTORY_ADDRESS,
  signerPrivateKey: EOA_PRIVATE_KEY,
  eoaValidatorAddress: EOA_VALIDATOR_ADDRESS,
});

const mainBundler = createBundlerClient({
  account: mainAccount,
  transport: http(BUNDLER_URL),
  chain: zkSyncSepoliaTestnet,
  client: publicClient,
});

// 2. Define and create session
const sessionSpec = {
  signer: '0x...' as Address, // Session signer address
  expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600), // 1 hour
  feeLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther('0.1'),
    period: 0n,
  },
  callPolicies: [],
  transferPolicies: [
    {
      target: '0x...' as Address,
      maxValuePerUse: parseEther('0.01'),
      valueLimit: {
        limitType: LimitType.Unlimited,
        limit: 0n,
        period: 0n,
      },
    },
  ],
};

const { userOpHash } = await createSessionAction(mainBundler, {
  sessionSpec,
  contracts: { sessionValidator: SESSION_VALIDATOR_ADDRESS },
});

await publicClient.waitForTransactionReceipt({ hash: userOpHash });

// 3. Use session for transactions
const sessionAccount = await createSessionSmartAccount(publicClient, {
  entryPoint: ENTRY_POINT_ADDRESS,
  factory: FACTORY_ADDRESS,
  accountAddress: await mainAccount.getAddress(),
  signerPrivateKey: SESSION_PRIVATE_KEY,
  sessionValidatorAddress: SESSION_VALIDATOR_ADDRESS,
  sessionSpec, // MUST match creation spec
});

const sessionBundler = createBundlerClient({
  account: sessionAccount,
  transport: http(BUNDLER_URL),
  chain: zkSyncSepoliaTestnet,
  client: publicClient,
});

// Send a transaction
const txHash = await sessionBundler.sendUserOperation({
  calls: [
    {
      to: sessionSpec.transferPolicies[0].target,
      value: parseEther('0.005'),
      data: '0x',
    },
  ],
});

console.log('Transaction sent:', txHash);
```

## Troubleshooting

### AA23 SessionNotActive

**Symptoms**: Transaction fails with `AA23 SessionNotActive` error.

**Causes**:

1. Session spec mismatch between creation and usage
2. Session not yet created on-chain (UserOp not confirmed)
3. Wrong session validator address
4. Session expired

**Solutions**:

```typescript
// ✅ Good: Store spec and reuse
const sessionSpec = { /* ... */ };
await createSessionAction(bundler, { sessionSpec });
const account = await createSessionSmartAccount(client, { sessionSpec });

// ❌ Bad: Different specs
const spec1 = { target: '0x000...000' };
const spec2 = { target: '0x123...456' }; // Different!

// ✅ Good: Wait for confirmation
const { userOpHash } = await createSessionAction(bundler, { sessionSpec });
await client.waitForTransactionReceipt({ hash: userOpHash });

// ❌ Bad: Use session immediately
await createSessionAction(bundler, { sessionSpec });
// UserOp might not be mined yet!
await sessionBundler.sendUserOperation({ /* ... */ });
```

### AA24 signature error

**Symptoms**: Transaction fails with `AA24 signature error`.

**Causes**:

1. Wrong session private key
2. Incorrect nonce calculation
3. Timestamp outside session validity

**Solutions**:

```typescript
// ✅ Verify private key matches signer address
const sessionAddress = privateKeyToAddress(SESSION_PRIVATE_KEY);
assert(sessionAddress === sessionSpec.signer);

// ✅ Check expiration
const now = Math.floor(Date.now() / 1000);
assert(now < sessionSpec.expiresAt);
```

### LifetimeUsageExceeded (0x5dd55ff1)

**Symptoms**: Transaction reverts with custom error `LifetimeUsageExceeded`.

**Causes**:

1. Fee limit exceeded
2. Value limit exceeded (transfer policies)
3. Value limit exceeded (call policies)

**Solutions**:

```typescript
// ✅ Monitor usage
const usedFees = await getSessionUsedFees(account, sessionId);
const remainingFees = sessionSpec.feeLimit.limit - usedFees;

// ✅ Set higher limits
const sessionSpec = {
  // ...
  feeLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther('2'), // Increased from 1 ETH
    period: 0n,
  },
};
```

## Best Practices

### 1. Secure Session Key Storage

```typescript
// ❌ Never commit private keys
const SESSION_KEY = '0xabcd...'; // BAD!

// ✅ Use environment variables
const SESSION_KEY = process.env.SESSION_PRIVATE_KEY;

// ✅ Or generate ephemeral keys
import { generatePrivateKey } from 'viem/accounts';
const ephemeralKey = generatePrivateKey();
```

### 2. Reasonable Expiration Times

```typescript
// ✅ Short-lived sessions for sensitive operations
const shortSession = {
  expiresAt: BigInt(Math.floor(Date.now() / 1000) + 900), // 15 minutes
  // ...
};

// ✅ Longer sessions for convenience features
const longSession = {
  expiresAt: BigInt(Math.floor(Date.now() / 1000) + 604800), // 7 days
  // ...
};

// ❌ Avoid excessively long sessions
const badSession = {
  expiresAt: BigInt(Math.floor(Date.now() / 1000) + 31536000), // 1 year - too long!
  // ...
};
```

### 3. Explicit Recipient Restrictions

```typescript
// ✅ Good: Specific recipient addresses
const transferPolicies = [
  {
    target: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    maxValuePerUse: parseEther('0.01'),
    // ...
  },
];

// ⚠️ Avoid wildcards if possible (implementation-specific)
// Some implementations may allow '0x0...0' as "any address"
```

### 4. Conservative Value Limits

```typescript
// ✅ Start with lower limits
const conservativePolicy = {
  maxValuePerUse: parseEther('0.001'), // Small per-transaction limit
  valueLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther('0.1'), // Small lifetime limit
    period: 0n,
  },
};

// ✅ Increase after testing
// Once you confirm usage patterns, adjust limits accordingly
```

### 5. Test Session Specs

```typescript
// ✅ Test session creation in development
async function testSession() {
  const spec = { /* your spec */ };
  
  // Create session
  const { userOpHash } = await createSessionAction(bundler, {
    sessionSpec: spec,
    contracts: { sessionValidator: SESSION_VALIDATOR_ADDRESS },
  });
  
  await client.waitForTransactionReceipt({ hash: userOpHash });
  
  // Verify session works
  const sessionAccount = await createSessionSmartAccount(client, {
    // ... use same spec ...
    sessionSpec: spec,
  });
  
  const testTx = await sessionBundler.sendUserOperation({
    calls: [{ to: spec.transferPolicies[0].target, value: 1n, data: '0x' }],
  });
  
  console.log('Test successful:', testTx);
}
```

## Advanced Topics

### Nonce Management

Sessions use a "keyed nonce" system where the nonce is derived from the session signer address:

```typescript
import { keyed_nonce_decimal } from '@zksync-sso/viem-session-sdk/bundler';

// Calculate the keyed nonce for a session signer
const keyedNonce = await keyed_nonce_decimal(
  sessionSignerAddress,
  entryPointAddress,
  accountAddress,
  publicClient
);

console.log('Keyed nonce:', keyedNonce);
```

This allows multiple sessions from different signers to operate independently without nonce conflicts.

### Custom Stub Signatures

For gas estimation, you can generate stub signatures without making real signatures:

```typescript
import { generate_session_stub_signature_wasm } from '@zksync-sso/viem-session-sdk/bundler';

const stubSig = await generate_session_stub_signature_wasm(
  sessionSpec,
  sessionValidatorAddress
);

// Use in gas estimation
const estimatedGas = await bundlerClient.estimateUserOperationGas({
  // ...
  signature: stubSig,
});
```

### Session Signature Without Validation

For testing or special cases, create signatures that skip time validation:

```typescript
import { session_signature_no_validation_wasm } from '@zksync-sso/viem-session-sdk/bundler';

const sig = await session_signature_no_validation_wasm(
  userOpHash,
  privateKey,
  accountAddress,
  sessionSpec,
  sessionValidatorAddress
);
```

**Warning**: Use only for testing! Production code should use normal session signatures with proper validation.

## Further Reading

- [examples/demo-app](../../../examples/demo-app) - Complete demo app with session flows
- [viem Account Abstraction Docs](https://viem.sh/account-abstraction) - Viem's AA documentation
- [ERC-4337 Specification](https://eips.ethereum.org/EIPS/eip-4337) - Official ERC-4337 spec
