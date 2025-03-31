# oidc-server

This package contains 2 services that are needed in order to use the OIDC
recovery solution for Single Sign On accounts.

1. **Salt service**: This is a service that generates a deterministic salt for a
   valid jwt.
2. **Contract updater**: Simple cron-like service that queries for the current
   public keys for the associated providers. When new keys are found they are
   stored in the `KeyRegistry` contract.

## How to run it

1. install dependencies:

```bash
pnpm install
```

2. Build

```bash
pnpm build
```

3. Configure

First you need to copy the configuration template, and then fill it with your
data:

```bash
cp example.env .env
```

```dotenv
FETCH_INTERVAL=60000
ADMIN_PRIVATE_KEY=0x.. #
NETWORK=mainnet        # Valid values: "mainnet", "sepolia" or "localhost"
CONTRACT_ADDRESS=0x... # Address for key registry contract
SALT_ENTROPY=0x0139201 # Secure random value.
APP_AUD= # client id
SALT_SERVICE_PORT=
AUTH_SERVER_URL=
```

2.To run `key-registry`:

```bash
pnpm dev:key-registry
```

To run `salt-service`:

```bash
pnpm dev:salt
```
