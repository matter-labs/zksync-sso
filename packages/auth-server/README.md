# zksync-sso-auth-server

ZKsync SSO Auth Server

## How to deploy locally?

```sh
# Ensure era_test_node is already running (npx zksync-cli dev start)
# Deploy ZKsync SSO smart contracts
pnpm nx deploy contracts

# Start Auth Server
pnpm nx dev auth-server
```

## Deploy

```sh
pnpm i

cp .env.example .env

# generate artifacts
nx run auth-server:build

# artifacts will be in ./dist

# serve the artifacts
npx serve ./dist
```
