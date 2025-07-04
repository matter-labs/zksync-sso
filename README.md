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
- 💻 Simple SDKs :
  - JavaScript
  - React Native
  - Swift
  - Kotlin _(Coming Soon)_
- 🤝 Open-source authentication server
- 🎓 Examples to get started quickly

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

This monorepo is comprised of the following packages, products, and examples:

- `packages/sdk` is the `zksync-sso` JavaScript SDK
- `packages/auth-server` is the Auth Server used for account creation and
  session key management
- `packages/contracts` are the on-chain smart contracts behind ZKsync SSO
  accounts
- `examples/nft-quest` is an app demonstrating the use of ZKsync SSO with
  sessions
- `examples/nft-quest-contracts` are the smart contracts for `nft-quest`
- `examples/demo-app` is a test app mostly used for CI testing
- `examples/bank-demo` is an app demonstrating the fully embedded experience

## Running development

1. Install workspace dependencies with PNPM.

   ```bash
   pnpm install
   ```

2. If creating new packages: use pnpm and
   [workspace protocol](https://pnpm.io/workspaces#workspace-protocol-workspace)
   to link SDK in the new folder.

3. Install `foundry-zksync`:

   ```bash
   curl -L https://raw.githubusercontent.com/matter-labs/foundry-zksync/main/install-foundry-zksync | bash
   ```

4. Start a local node:

   ```bash
   npx zksync-cli dev start
   ```

5. Compile and deploy contracts to the local node:

   ```bash
   # Compile and deploy contracts
   pnpm --dir packages/contracts run build
   pnpm --dir packages/contracts run publish ../sdk/src/abi
   pnpm run format-abi
   pnpm --dir packages/contracts run deploy --file ../auth-server/stores/local-node.json
   ```

6. Start the demo application:

   ```bash
   pnpm nx dev demo-app
   ```

Your local Auth Server will be running at `http://localhost:3002/`, and the demo
app will be running at `http://localhost:3004/`.

## Running commands

Use the NX CLI to run project commands, however PNPM is still usable as an
alternative. NX project names are based on the name defined in each project's
`project.json` which are set to match the directory name.

```bash
pnpm nx <target> <project>
# Example
pnpm nx build sdk
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

To execute the end-to-end tests for the `demo-app` (or similarly for
`nft-quest`), you'll need to do some setup:

1. Start `era_test_node` (In a separate terminal, run
   `npx zksync-cli dev start`)
2. Deploy the smart contracts, `pnpm --dir packages/contracts run deploy`

Once the local node is configured with the smart contracts deployed, you can run
the e2e tests:

```bash
pnpm nx e2e demo-app
```

To debug the end-to-end tests:

```bash
pnpm nx e2e:debug demo-app
```
