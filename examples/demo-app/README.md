# Demo App

This app demonstrates a basic use of the ZKsync SSO SDK and provides e2e test
coverage for the SDK.

## Requirements

You will need the local `zksync-os` stack and local bundler processes.

From the repository root, start it with:

```bash
pnpm dev:stack:up
```

Then start the bundler processes in separate terminals:

```bash
pnpm --dir packages/contracts run bundler
```

```bash
pnpm --dir packages/contracts run bundler-proxy
```

## Setup

Run the following commands from the root of the monorepo.

```bash
pnpm install
```

Running the Demo App requires the Auth Server. The local target below also
deploys the local contract suite, bridges the small set of local wallets needed
for testing, and deploys the mock paymaster.

```bash
pnpm nx dev demo-app
```

The output will list the localhost addresses for both running applications.
