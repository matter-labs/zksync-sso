# zksync-sso-web-sdk

**⚠️ WORK IN PROGRESS**: This package is currently under active
development and not yet ready for production use.

WebAssembly-powered SDK for zkSync SSO ERC-4337 integration in web applications.

## Current Status

This package provides TypeScript bindings for a
Rust/WebAssembly implementation of zkSync SSO ERC-4337 functionality.

**What's Working:**

- ✅ WASM module builds successfully for bundler and Node.js targets
- ✅ TypeScript client wrapper with proper exports
- ✅ **Account deployment** with EOA validators
- ✅ **EOA-signed transactions** via ERC-4337 UserOperations
- ✅ Integration with Pimlico/Alto bundler APIs
- ✅ Gas estimation and UserOperation construction
- ✅ HTTP transport layer using `reqwasm` for WASM compatibility
- ✅ Comprehensive error handling

**What's In Progress:**

- ⏳ WebAuthn passkey integration (deployment works, transaction signing in testing)
- ⏳ Full end-to-end testing with hardware security keys
- ⏳ Session key validator integration
- ⏳ Multi-chain support
- ⏳ NPM package publishing

The SDK is **functional** and supports real ERC-4337 operations including
account deployment and EOA-signed transactions.

## Quick Start with Viem (Recommended)

For most use cases, the viem-based session API provides the easiest integration:

```typescript
import { toSmartAccount } from 'viem/account-abstraction';
import { 
  getAccountInitCode, 
  encodeSessionExecuteCallData,
  generateSessionStubSignature 
} from '@zksync-sso/viem-session-sdk/bundler';

// 1. Create a smart account with session support
const account = await toSmartAccount({
  client: publicClient,
  entryPoint: {
    address: ENTRY_POINT_ADDRESS,
    version: '0.8',
  },
  async getFactoryArgs() {
    return {
      factory: FACTORY_ADDRESS,
      factoryData: await getAccountInitCode(userId, [eoaSigner]),
    };
  },
  async encodeCallData(calls) {
    // Use session-based encoding for transaction execution
    return await encodeSessionExecuteCallData(
      calls,
      sessionSpec,
      sessionValidatorAddress
    );
  },
  async getStubSignature() {
    return await generateSessionStubSignature(sessionSpec);
  },
});

// 2. Create bundler client
const bundlerClient = createBundlerClient({
  account,
  transport: http(BUNDLER_URL),
  chain,
  client: publicClient,
});

// 3. Send transactions with automatic session handling
const userOpHash = await bundlerClient.sendUserOperation({
  calls: [
    {
      to: recipientAddress,
      value: parseEther('0.001'),
      data: '0x',
    },
  ],
});
```

### Session Specifications

Sessions define time-limited, scoped permissions for transaction signing:

```typescript
interface SessionSpec {
  signer: Address;           // Address authorized to use this session
  expiresAt: bigint;         // Unix timestamp when session expires
  feeLimit: {
    limitType: LimitType;    // Unlimited | Lifetime | Allowance
    limit: bigint;           // Max fee in wei
    period: bigint;          // Period for Allowance type
  };
  callPolicies: CallPolicy[];        // Smart contract call restrictions
  transferPolicies: TransferPolicy[]; // ETH transfer restrictions
}
```

**Critical**: The session spec used when creating a session **must exactly
match** the spec used when sending transactions, or you'll get
`AA23 SessionNotActive` errors.

### Complete Session Example

```typescript
// Define session parameters
const sessionSpec = {
  signer: sessionSignerAddress,
  expiresAt: BigInt(Math.floor(Date.now() / 1000) + 86400), // 24 hours
  feeLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther('1'), // Max 1 ETH in fees
    period: 0n,
  },
  callPolicies: [],
  transferPolicies: [
    {
      target: allowedRecipient,
      maxValuePerUse: parseEther('0.001'), // Max 0.001 ETH per tx
      valueLimit: {
        limitType: LimitType.Unlimited,
        limit: 0n,
        period: 0n,
      },
    },
  ],
};

// Create session on-chain (using EOA-controlled account)
const createResult = await createSessionAction(eoaBundlerClient, {
  sessionSpec,
  contracts: { sessionValidator: SESSION_VALIDATOR_ADDRESS },
});

// Use session for transactions (using session-controlled account)
const sendResult = await sessionBundlerClient.sendUserOperation({
  calls: [
    { to: allowedRecipient, value: parseEther('0.001'), data: '0x' },
  ],
});
```

See [examples/demo-app](../../examples/demo-app) for complete working examples
with session creation and usage.

## Documentation

- **[Session API Guide](./docs/session-api.md)** - Comprehensive guide to
  session-based transactions
- **[Migration Guide](./docs/migration-guide.md)** - Migrating from low-level
  WASM API to viem
- **API Reference** - See inline JSDoc comments in TypeScript files
- **Examples** - See [examples/demo-app](../../examples/demo-app) for working
  code

## Installation

This is a beta development package.

To build locally:

```bash
cd packages/sdk-platforms/web
npm install
npm run build
```

## Key Features

### Local Development Setup

The SDK is designed to work with a local Anvil node and Alto bundler for development:

```bash
# Terminal 1: Start local Anvil node
anvil

# Terminal 2: Deploy contracts
cd packages/erc4337-contracts
pnpm deploy:local

# Terminal 3: Start Alto bundler
cd examples/demo-app
pnpm dev:bundler
```

This creates a local environment at:

- **RPC**: `http://localhost:8545` (Anvil)
- **Bundler**: `http://localhost:4337` (Alto via bundler-proxy)
- **Entry Point**: `0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108`

### Account Deployment

Deploy ERC-4337 modular smart accounts with EOA validators:

```typescript
import { deploy_account, DeployAccountConfig } from 'zksync-sso-web-sdk/bundler';

const config = new DeployAccountConfig(
  "http://localhost:8545",           // RPC URL
  "0xBF0c79fD7b3eeF6Ac1bf67CEef3B2adED40ddC40",  // Factory address (from deploy:local)
  "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",  // Deployer private key (Anvil account #0)
  "0x3a09a0f6Cb773BCfdf606D21FAecFe5699f74718",  // EOA validator address
  null  // WebAuthn validator (optional, pass address if using passkeys)
);

// Deploy with EOA signer (Anvil account #1)
const accountAddress = await deploy_account(
  "user-123",  // User ID
  ["0x70997970C51812dc3A010C7d01b50e0d17dc79C8"],  // EOA signer addresses
  null,  // No passkey payload for basic deployment
  config
);

console.log("Account deployed at:", accountAddress);
```

### EOA-Signed Transactions

Send transactions signed by an EOA private key via ERC-4337:

```typescript
import { send_transaction_eoa, SendTransactionConfig } from 'zksync-sso-web-sdk/bundler';

const txConfig = new SendTransactionConfig(
  "http://localhost:8545",           // RPC URL
  "http://localhost:4337",           // Bundler URL (via bundler-proxy)
  "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108"  // Entry Point address
);

const receipt = await send_transaction_eoa(
  txConfig,
  "0x3a09a0f6Cb773BCfdf606D21FAecFe5699f74718",  // EOA validator address
  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",  // Private key (Anvil account #1)
  accountAddress,                     // Your smart account
  "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",  // Recipient
  "1000000000000000000",             // 1 ETH in wei
  null                                // Optional call data
);

console.log("Transaction receipt:", receipt);
```

### Configuration from contracts.json

The demo app loads configuration from a generated file:

```typescript
// Load deployed contract addresses
const response = await fetch("/contracts.json");
const contracts = await response.json();

const config = new DeployAccountConfig(
  contracts.rpcUrl,           // "http://localhost:8545"
  contracts.factory,          // Deployed factory address
  deployerPrivateKey,
  contracts.eoaValidator,     // Deployed EOA validator
  contracts.webauthnValidator // Deployed WebAuthn validator (if using passkeys)
);
```

## API Reference

### Account Deployment

#### deploy_account(userId, eoaSigners, passkeyPayload, config)

Deploys a modular smart account with optional validators.

**Parameters:**

- `userId: string` - Unique user identifier
- `eoaSigners: string[]` - Array of EOA addresses to authorize as signers
- `passkeyPayload: PasskeyPayload | null` - Optional passkey configuration
- `config: DeployAccountConfig` - Deployment configuration

**Returns:** `Promise<string>` - Deployed account address

### Transaction Functions

#### send_transaction_eoa

Send a transaction signed by an EOA private key.

**Parameters:**

- `config: SendTransactionConfig` - RPC and bundler configuration
- `validatorAddress: string` - EOA validator contract address
- `privateKey: string` - Hex-encoded private key (with 0x prefix)
- `accountAddress: string` - Smart account address
- `toAddress: string` - Recipient address
- `value: string` - Wei amount as string
- `data: string | null` - Optional call data (hex with 0x prefix, or null)

**Returns:** `Promise<string>` - Transaction receipt hash

### Configuration Types

#### DeployAccountConfig

```typescript
class DeployAccountConfig {
  constructor(
    rpc_url: string,
    account_factory_address: string,
    deployer_private_key: string,
    eoa_validator_address: string,
    webauthn_validator_address: string | null
  )
}
```

Configuration for deploying smart accounts:

- `rpc_url` - Ethereum RPC endpoint (e.g., `http://localhost:8545`)
- `account_factory_address` - Factory contract that deploys accounts
- `deployer_private_key` - Private key used to fund deployment
- `eoa_validator_address` - EOA validator contract address
- `webauthn_validator_address` - WebAuthn validator address (null if not using passkeys)

#### SendTransactionConfig

```typescript
class SendTransactionConfig {
  constructor(
    rpc_url: string,
    bundler_url: string,
    entry_point_address: string
  )
}
```

Configuration for sending transactions:

- `rpc_url` - Ethereum RPC endpoint
- `bundler_url` - ERC-4337 bundler endpoint (e.g., `http://localhost:4337`)
- `entry_point_address` - EntryPoint v0.8 contract address

### Default Addresses (Local Development)

When using `pnpm deploy:local` in the erc4337-contracts package:

```typescript
const ENTRY_POINT = "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108";  // EntryPoint v0.8
// Other addresses available in contracts.json after deployment
```

### Anvil Test Accounts

```typescript
// Account #0 (Deployer)
const DEPLOYER_KEY = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const DEPLOYER_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

// Account #1 (EOA Signer)
const SIGNER_KEY = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const SIGNER_ADDRESS = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
```

## Building from Source

### Prerequisites

- Node.js 20+
- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack` CLI tool (`cargo install wasm-pack`)
- Foundry (for ERC-4337 contracts)

### Build Steps

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Build WASM modules for both bundler and Node.js
cd packages/sdk-platforms/web
pnpm install
pnpm run build
```

This will:

1. Build the Rust WASM module for bundler target (`pkg-bundler/`)
2. Build the Rust WASM module for Node.js target (`pkg-node/`)
3. Compile TypeScript wrapper and generate type definitions (`dist/`)

### Local Development Environment

Set up a complete local development environment:

```bash
# Terminal 1: Start Anvil
anvil

# Terminal 2: Deploy contracts
cd packages/erc4337-contracts
pnpm deploy:local

# Terminal 3: Start bundler via proxy
cd examples/demo-app
pnpm dev:bundler

# Terminal 4: Run demo app
cd examples/demo-app
pnpm dev
```

Then navigate to `http://localhost:3000/web-sdk-test` to test the SDK.

### Testing

Run the test suite:

```bash
pnpm test
```

## Architecture

The SDK is built with a Rust core compiled to WebAssembly:

```mermaid
┌─────────────────────────────────────────┐
│   TypeScript Application Layer          │
│  (bundler.ts exports)                   │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│   WASM FFI Bindings (wasm-bindgen)      │
│  (zksync-sso-erc4337-ffi-web crate)     │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│   Rust Core Library                     │
│  (zksync-sso-erc4337-core crate)        │
│  - Account deployment                   │
│  - UserOperation construction           │
│  - Signature generation                 │
│  - Gas estimation                       │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│   Alloy (Ethereum library)              │
│  + Custom WASM HTTP Transport           │
│  (reqwasm for browser compatibility)    │
└─────────────────────────────────────────┘
```

### Key Components

- **WASM Transport Layer**: Custom HTTP transport using `reqwasm` for browser compatibility
- **Bundler Integration**: Direct integration with Alto bundler via bundler-proxy
- **Signature Providers**: Pluggable signature generation for EOA validators
- **Type Safety**: Full TypeScript type definitions generated from Rust

## Development Roadmap

- [x] Core ERC-4337 account deployment
- [x] EOA validator integration
- [x] Bundler API integration (Alto via bundler-proxy)
- [x] Gas estimation
- [x] Local development environment with Anvil + Alto
- [ ] WebAuthn passkey transaction signing (in progress)
- [ ] Session key validator support
- [ ] Multi-chain configuration
- [ ] Comprehensive E2E tests
- [ ] Performance optimizations
- [ ] NPM package publishing
- [ ] Documentation site

## Related Packages

- [`@zksync-sso/sdk`](../../sdk/) - Main TypeScript SDK (production-ready)
- [`@zksync-sso/contracts`](../../contracts/) - Smart contracts
- [`zksync-sso-erc4337-core`](../rust/zksync-sso-erc4337/) - Rust core library

## License

Apache License 2.0 - see the [LICENSE](../../../LICENSE-APACHE) file for
details.
