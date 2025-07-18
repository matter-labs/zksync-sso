# zksync-sso SDK

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE-MIT)

A user & developer friendly modular smart account implementation on ZKsync;
simplifying user authentication, session management, and transaction processing.

## Getting started

Install the ZKsync SSO SDK package:

```sh
npm i zksync-sso-wagmi-connector
# optional peer dependencies
npm i @simplewebauthn/browser @simplewebauthn/server @wagmi/core
```

Optional peer dependencies that you may need to install based on your usage:

- `@simplewebauthn/browser` and `@simplewebauthn/server` (v13.x) - Required for
  passkey operations
- `@wagmi/core` (v2.x) - Required for using the SSO connector

Add ZKsync SSO connector to your app (using `wagmi`):

[Find more information here in our docs.](https://docs.zksync.io/build/zksync-sso)
