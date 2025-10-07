# zksync-sso-web-sdk

**⚠️ WORK IN PROGRESS**: This package is currently under active
development and not yet ready for production use.

WebAssembly-powered SDK for zkSync SSO ERC-4337 integration in web applications.

## Current Status

This package provides TypeScript bindings for a
Rust/WebAssembly implementation of zkSync SSO ERC-4337 functionality.

**What's Working:**

- ✅ WASM module builds successfully
- ✅ TypeScript client wrapper
- ✅ Basic type definitions
- ✅ Multiple platform targets (bundler, Node.js)

**What's Not Yet Implemented:**

- ⏳ Actual user operation submission (currently returns mock hash)
- ⏳ Integration with bundler APIs
- ⏳ Transaction signing
- ⏳ Gas estimation
- ⏳ Account deployment
- ⏳ Error handling and validation

The current implementation is a **stub**
that demonstrates the architecture but does not yet perform real ERC-4337 operations.

## Installation

**Not yet published to NPM.** This is a development package.

To build locally:

```bash
cd packages/sdk-platforms/web
npm install
npm run build
```

## Usage Example

```typescript
import { ZkSyncSsoClient } from "./packages/sdk-platforms/web/dist/index.js";

const config = {
  rpcUrl: "https://sepolia.era.zksync.dev",
  bundlerUrl: "https://bundler.example.com",
  contracts: {
    entryPoint: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
    accountFactory: "0x9406Cc6185a346906296840746125a0E44976454",
  },
};

const client = new ZkSyncSsoClient(config, "0x1234..."); // Your private key

// Note: This currently returns a mock hash, not a real transaction
const result = await client.sendUserOperation({
  account: "0xabc...",
  calls: [
    {
      to: "0xdef...",
      data: "0x...",
      value: "1000000000000000000",
    },
  ],
});

console.log("Mock hash:", result);
// Outputs: "0x1234567890abcdef..."
```

## API Reference

### ZkSyncSsoClient

#### Constructor

```typescript
constructor(config: ClientConfig, privateKey: string)
```

Creates a client instance (currently stub implementation).

#### Methods

##### sendUserOperation(request: UserOperationRequest): Promise\<string\>

**⚠️ Stub Implementation**: Currently logs the request
and returns a mock transaction hash.

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
  to: string;
  data: string;
  value: string;
}
```

## Building from Source

### Prerequisites

- Node.js 18+
- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack` CLI tool
- Foundry (for building ERC-4337 contracts)

### Build Steps

```bash
# Build ERC-4337 contracts first (required for WASM compilation)
cd packages/erc4337-contracts
forge soldeer install
forge build

# Build WASM modules and TypeScript
cd ../sdk-platforms/web
npm install
npm run build
```

## Development Roadmap

- [ ] Implement actual bundler communication
- [ ] Add transaction signing with private key
- [ ] Implement gas estimation
- [ ] Add account deployment support
- [ ] Proper error handling
- [ ] Add comprehensive tests
- [ ] Add documentation
- [ ] Publish to NPM

## Related Packages

- [`@zksync-sso/sdk`](../../sdk/) - Main TypeScript SDK (production-ready)
- [`@zksync-sso/contracts`](../../contracts/) - Smart contracts
- [`zksync-sso-erc4337-core`](../rust/zksync-sso-erc4337/) - Rust core library

## License

Apache License 2.0 - see the [LICENSE](../../../LICENSE-APACHE) file for
details.
