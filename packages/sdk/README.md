# zksync-sso SDK

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE-MIT)

A user & developer friendly modular smart account implementation on ZKsync;
simplifying user authentication, session management, and transaction processing.

## Features and Goals

- 🧩 Modular smart accounts based on
  [ERC-7579](https://eips.ethereum.org/EIPS/eip-7579#modules)
- 🔑 Passkey authentication (no seed phrases)
- ⏰ Sessions with easy configuration and management
- 💰 Integrated paymaster support
- ❤️‍🩹 Account recovery
- 💻 Simple SDKs : JavaScript, iOS/Android _(Coming Soon)_
- 🤝 Open-source authentication server
- 🎓 Examples to get started quickly

## What is included?

- **Full SDK** (`zksync-sso`):

  - Complete smart account and session management for ZKsync.
  - Utilities for authentication, paymasters, and advanced session flows.
  - All types, helpers, and utilities for building custom integrations.
  - Includes the SSO wagmi connector for easy integration with wagmi/react apps.
  - Recommended for most app and wallet developers who need full control and
    advanced features.

- **Connector SDK** (`@zksync-sso/connector-lite`):
  - Minimal, standalone wagmi connector for ZKsync SSO.
  - Lightweight: only the code and dependencies required for wallet connection.
  - Ideal for apps that only need to connect to ZKsync SSO via wagmi, without
    extra SDK features.
  - Smaller bundle size and fewer dependencies.

## Getting started

Install the ZKsync SSO SDK package:

```sh
npm i zksync-sso
# optional peer dependencies
npm i @simplewebauthn/browser @simplewebauthn/server @wagmi/core
```

Optional peer dependencies that you may need to install based on your usage:

- `@simplewebauthn/browser` and `@simplewebauthn/server` (v13.x) - Required for
  passkey operations
- `@wagmi/core` (v2.x) - Required for using the SSO connector

If you only need the minimal wagmi connector, install the lite package:

```sh
npm i @zksync-sso/connector-lite
npm i @wagmi/core
```

## Documentation

- [Full SDK and advanced usage](https://docs.zksync.io/build/zksync-sso)
- [Wagmi connector quickstart](https://docs.zksync.io/build/zksync-sso/connector)
