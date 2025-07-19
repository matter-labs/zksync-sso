# ZKsync SSO Wagmi Connector

A lightweight wagmi connector for ZKsync SSO integration. This package contains
only the wagmi connector functionality for users who want to integrate ZKsync
SSO into their wagmi-based applications without the full SDK.

## Installation

```bash
npm install @zksync-sso/connector-lite
# or
yarn add @zksync-sso/connector-lite
# or
pnpm add @zksync-sso/connector-lite
```

## Usage

```typescript
import { createConfig } from "@wagmi/core";
import { zksyncSsoConnector } from "zksync-sso-wagmi-connector";
import { zksyncSepoliaTestnet } from "viem/chains";

const ssoConnector = zksyncSsoConnector({
  metadata: {
    name: "My App",
    icon: "https://example.com/icon.png",
  },
  authServerUrl: "https://your-auth-server.com/confirm",
  session: {
    feeLimit: parseEther("0.1"),
    transfers: [
      {
        to: "0x...",
        valueLimit: parseEther("0.05"),
      },
    ],
  },
});

const config = createConfig({
  chains: [zksyncSepoliaTestnet],
  connectors: [ssoConnector],
  // ... other config
});
```

## Features

- **Lightweight**: Only includes the wagmi connector functionality
- **Type-safe**: Full TypeScript support
- **Session management**: Built-in session handling for gasless transactions
- **Flexible**: Support for custom paymaster handlers and session policies

## Peer Dependencies

This package requires `@wagmi/core` version 2.x as a peer dependency.

## Full SDK

If you need the complete ZKsync SSO SDK with additional features like direct
client usage, use the main `zksync-sso` package instead.

## Documentation

For detailed documentation and examples, visit
[ZKsync SSO Documentation](https://docs.zksync.io/build/zksync-sso).

## License

MIT
