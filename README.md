# ZKsync SSO

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE-MIT)
[![CI](https://github.com/matter-labs/zksync-account-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/matter-labs/zksync-account-sdk/actions/workflows/ci.yml)

A user & developer friendly modular smart account implementation on ZKsync;
simplifying user authentication, session management, and transaction processing.

## Features and Goals

<!-- prettier-ignore -->
> [!CAUTION]
ZKsync SSO is under active development and not yet feature complete. While it's
suitable for improving your development workflows and tooling,
use caution and stay up to date with the latest updates and changes as they are released.

- 🧩 Modular smart accounts based on
  [ERC-7579](https://eips.ethereum.org/EIPS/eip-7579#modules)
- 🔑 Passkey authentication (no seed phrases)
- ⏰ Sessions with easy configuration and management
- 💰 Integrated paymaster support
- ❤️‍🩹 Account recovery
  - Setup EOA or other SSO Guardian accounts to create new passkey
  - Use Google Auth (OIDC) recovery flow to create new passkey
- 💻 JavaScript SDK for ERC-4337 smart accounts on zksync-os
- 🤝 Open-source authentication server
- 🎓 Demo app to get started quickly

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

Add ZKsync SSO connector to your app (using `wagmi`):

```ts
import { zksyncSsoConnector, callPolicy } from "zksync-sso/connector";
import { zksyncSepoliaTestnet } from "viem/chains";
import { createConfig, connect } from "@wagmi/core";
import { erc20Abi } from "viem";

const ssoConnector = zksyncSsoConnector({
   // Optional session configuration, if omitted user will have to sign every transaction via Auth Server
   session: {
      expiry: "1 day",

      // Allow up to 0.1 ETH to be spend in gas fees
      feeLimit: parseEther("0.1"),

      transfers: [
         // Allow ETH transfers of up to 0.1 ETH to specific address
         {
            to: "0x188bd99cd7D4d78d4E605Aeea12C17B32CC3135A",
            valueLimit: parseEther("0.1"),
         },
      ],

      // Allow calling specific smart contracts (e.g. ERC20 transfer):
      contractCalls: [
         callPolicy({
            address: "0xa1cf087DB965Ab02Fb3CFaCe1f5c63935815f044",
            abi: erc20Abi,
            functionName: "transfer",
            constraints: [
               // Only allow transfers to this address. Or any address if omitted
               {
                  index: 0, // First argument of erc20 transfer function, recipient address
                  value: "0x6cC8cf7f6b488C58AA909B77E6e65c631c204784",
               },

               // Allow transfering up to 0.2 tokens per hour
               // until the session expires
               {
                  index: 1,
                  limit: {
                     limit: parseUnits("0.2", TOKEN.decimals),
                     period: "1 hour",
                  },
               },
            ],
         }),
      ],
   },

   // Optional: Receive notifications about session state changes
   onSessionStateChange: ({ state, address, chainId }) => {
      console.log(`Session state for address ${address} changed: ${state.type} - ${state.message}`);

      // Use this to notify users and restart the session if needed
      // - Session expired: state.type === 'session_expired'
      // - Session inactive (e.g. was revoked): state.type === 'session_inactive'
   },
});

const wagmiConfig = createConfig({
   connectors: [ssoConnector],
   ..., // your wagmi config https://wagmi.sh/core/api/createConfig
});

const connectWithSSO = () => {
   connect(wagmiConfig, {
      connector: ssoConnector,
      chainId: zksyncSepoliaTestnet.id, // or another chain id that has SSO support
   });
};
```

[Find more information here in our docs.](https://docs.zksync.io/build/zksync-sso)

## Local Development

This monorepo is comprised of the active ERC-4337 development path:

- `packages/sdk-4337` is the `zksync-sso` JavaScript SDK
- `packages/sdk-platforms/web` provides the Rust/WASM web bindings used by the
  SDK
- `packages/auth-server` is the Auth Server used for account creation and
  session key management
- `packages/erc4337-contracts` are the on-chain smart contracts behind ZKsync
  SSO accounts
- `packages/bundler` contains the Alto-facing bundler helpers and config
- `examples/demo-app` is the local integration app for the current stack

## Prerequisites

Install these once before running the local development flow:

- Node.js 22.x
- `pnpm`
- [Foundry](https://getfoundry.sh)

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

## Running development

1. Install dependencies.

   ```bash
   pnpm install
   ```

2. Create .env files (first time only):

   ```bash
   cp packages/auth-server/.env.example packages/auth-server/.env
   cp packages/auth-server-api/.env.example packages/auth-server-api/.env
   cp packages/bundler/.env.example packages/bundler/.env
   ```

3. Install ERC-4337 Soldeer dependencies:

   ```bash
   cd packages/erc4337-contracts && forge soldeer install && cd ../..
   ```

4. Start the local `zksync-os` stack in a separate terminal:

   ```bash
   pnpm dev:stack:up
   ```

   This also predeploys the local bundler prerequisites.

5. Start the Alto bundler and CORS proxy in separate terminals:

   ```bash
   # Terminal 1
   pnpm --dir packages/erc4337-contracts run bundler
   ```

   ```bash
   # Terminal 2
   pnpm --dir packages/erc4337-contracts run bundler-proxy
   ```

6. Start the demo application and local SSO helpers:

   ```bash
   pnpm nx dev:erc4337 demo-app
   ```

   This uses `scripts/setup-local-dev.sh`.

For reusable deployments, use `scripts/deploy-sso-contracts.sh`.

Local port list:

- auth server: 3002
- auth server api: 3004
- demo app: 3005
- bundler CORS proxy: 4337
- Alto bundler: 4338
- zksync-os L1: 5010
- zksync-os RPC: 3050

## Running commands

Use the NX CLI to run project commands, however PNPM is still usable as an
alternative. NX project names are based on the name defined in each project's
`project.json` which are set to match the directory name.

```bash
pnpm nx <target> <project>
# Example
pnpm nx build sdk-4337
```

To run a command in multiple projects, use the `run-many` command.

```bash
pnpm nx run-many -t <target> --all           # for all projects
pnpm nx run-many -t <target> -p proj1 proj2  # by project
pnpm nx run-many --targets=lint,test,build   # run multiple commands
```

Some commands are inferred and built-in with NX, thus you may not see commands
available from via the `package.json`. To review all the available commands in a
project:

```bash
pnpm nx show project <project> --web
```

## Lint project

At the root level of the monorepo, run the `pnpm run lint` command to run
linting across the project.

To fix lint issues that come up from linting, run the `pnpm run lint:fix`
command.

## Running/Debugging End-to-End Tests

To execute the end-to-end tests, complete steps 1–5 from "Running development"
above (zksync-os running + bundler running), then:

```bash
# ERC-4337 e2e tests
pnpm nx e2e:erc4337 demo-app

# Demo-only tests (session + passkey)
pnpm nx e2e:demo-only demo-app

# Guardian e2e tests
pnpm nx e2e:guardian auth-server
```

`pnpm nx e2e:erc4337 demo-app` runs `scripts/setup-local-dev.sh`, but still
expects `zksync-os`, Alto, and the bundler proxy to already be running.

To debug end-to-end tests interactively:

```bash
pnpm nx e2e:debug demo-app
```
