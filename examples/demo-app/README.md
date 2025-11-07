# Demo App

This app demonstrates a basic use of the ZKsync SSO SDK and provides e2e test
coverage for the SDK.

## Requirements

You will need [Era In Memory Node](https://github.com/matter-labs/era-test-node)
for deploying contracts locally.

In a terminal, start up the Era In Memory Node with the command `era_test_node`.

## Setup

Run the following commands from the root of the monorepo.

```bash
pnpm install
```

Running the Demo App requires the Auth Server. The following will start both.

```bash
pnpm nx dev demo-app
```

The output will list the localhost addresses for both running applications.

## Network configuration (contracts.json)

The demo app reads network and validator addresses from `/public/contracts.json`
at runtime. Ensure the file contains the following fields:

- `rpcUrl`: JSON-RPC endpoint for the local node
- `bundlerUrl`: ERC-4337 bundler URL
- `entryPoint`: EntryPoint contract address
- `factory`: Account factory address
- `eoaValidator`: EOA validator address
- `webauthnValidator`: WebAuthn validator address (for passkey flows)
- `sessionValidator`: SessionKey validator address (for session flows)

Session flows in the Web SDK (deploy-with-session and post-deploy install)
require `sessionValidator` to be set.
