# @zksync-sso/web-sdk

WebAssembly-powered SDK for zkSync SSO ERC-4337 integration in web applications.

## Features

- 🚀 **High Performance**: Built with Rust and WebAssembly for optimal performance
- 🌐 **Universal**: Works in both browsers and Node.js environments
- 🔒 **Type Safe**: Full TypeScript support with comprehensive type definitions
- 📦 **Multiple Entry Points**: Optimized builds for different environments
- 🛠 **ERC-4337 Ready**: Full support for account abstraction and user operations

## Installation

```bash
npm install @zksync-sso/web-sdk
```

## Usage

### Basic Usage

```typescript
import { ZkSyncSsoClient } from "@zksync-sso/web-sdk";

const config = {
  rpcUrl: "https://sepolia.era.zksync.dev",
  bundlerUrl: "https://bundler.example.com",
  contracts: {
    entryPoint: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
    accountFactory: "0x9406Cc6185a346906296840746125a0E44976454",
  },
};

const client = new ZkSyncSsoClient(config, "0x1234..."); // Your private key

// Send a user operation
const result = await client.sendUserOperation({
  account: "0xabc...", // Smart contract account address
  calls: [
    {
      to: "0xdef...",
      data: "0x...", 
      value: "1000000000000000000", // 1 ETH in wei
    },
  ],
});

console.log("User operation hash:", result);
```

### Environment-Specific Usage

#### For Bundlers (Webpack, Vite, etc.)

```typescript
import { ZkSyncSsoClient } from "@zksync-sso/web-sdk/bundler";
// Optimized for browser environments
```

#### For Node.js

```typescript
import { ZkSyncSsoClient } from "@zksync-sso/web-sdk/node";
// Optimized for Node.js environments  
```

### Advanced Usage

```typescript
import { 
  ZkSyncSsoClient, 
  ZkSyncSsoUtils,
  type ClientConfig,
  type UserOperationRequest 
} from "@zksync-sso/web-sdk";

// Utility functions
const isValidAddress = ZkSyncSsoUtils.isValidAddress("0x...");
const bytes = ZkSyncSsoUtils.hexToBytes("0xdeadbeef");
const hex = ZkSyncSsoUtils.bytesToHex(new Uint8Array([1, 2, 3]));

// Access the underlying WASM client for advanced features
const wasmClient = client.getWasmClient();
```

## API Reference

### ZkSyncSsoClient

#### Constructor

```typescript
constructor(config: ClientConfig, privateKey: string)
```

- `config`: Client configuration object
- `privateKey`: Private key for signing (hex string with or without 0x prefix)

#### Methods

##### sendUserOperation(request: UserOperationRequest): Promise<string>

Send a user operation to the bundler.

- `request`: User operation request containing account and calls
- Returns: Promise resolving to the user operation hash

### Types

#### ClientConfig

```typescript
interface ClientConfig {
  rpcUrl: string;
  bundlerUrl: string;
  contracts: {
    entryPoint: string;
    accountFactory: string;
  };
}
```

#### UserOperationRequest

```typescript
interface UserOperationRequest {
  account: string;
  calls: CallData[];
}
```

#### CallData

```typescript
interface CallData {
  to: string;      // Target contract address
  data: string;    // Transaction data (hex string)
  value: string;   // ETH value to send (in wei as string)
}
```

## Building from Source

### Prerequisites

- Node.js 18+
- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack` CLI tool

### Build Steps

```bash
# Install dependencies
npm install

# Build WASM modules
npm run build:wasm

# Build TypeScript
npm run build:ts

# Or build everything
npm run build
```

### Development

```bash
# Watch TypeScript changes
npm run dev

# Run tests
npm run test

# Type checking
npm run typecheck

# Linting
npm run lint
```

## Contributing

Please read our [Contributing Guide](../../../CONTRIBUTING.md)
for details on our code of conduct and the process for submitting
pull requests.

## License

This project is licensed under the
Apache License 2.0 - see the [LICENSE](../../../LICENSE-APACHE)
file for details.

## Related Packages

- [`@zksync-sso/sdk`](../../sdk/) - Main TypeScript SDK
- [`@zksync-sso/contracts`](../../contracts/) - Smart contracts
- [`@zksync-sso/circuits`](../../circuits/) - Zero-knowledge circuits
