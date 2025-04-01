# oidc-server

This package contains 2 services that are needed in order to use the OIDC
recovery solution for Single Sign On accounts.

1. **Salt service**: This is a service that generates a deterministic salt for a
   valid jwt.
2. **Contract updater**: Simple cron-like service that queries for the current
   public keys for the associated providers. When new keys are found they are
   stored in the `KeyRegistry` contract.

## How to run it

### Install dependencies

```bash
pnpm install
```

### Build

```bash
pnpm build
```

### Configure

First you need to copy the configuration template:

```bash
cp example.env .env
```

After that the templated can be filled. Here there is a little guide.

```dotenv
FETCH_INTERVAL=60000
ADMIN_PRIVATE_KEY=0x.. #
NETWORK=mainnet        # Valid values: "mainnet", "sepolia" or "localhost"
CONTRACT_ADDRESS=0x... # Address for key registry contract
SALT_ENTROPY=0x0139201 # Secure random value.
APP_AUD=               # client id
SALT_SERVICE_PORT=3003 # Port used by salt service
AUTH_SERVER_URL=       # Url for auth server. This is used to correctly configured cors. 
```

### Salt entropy

This value has to be a securely random generated value, and it has to be
securely saved as if it were a private key.

If you are running the service by the first time, the salt can be generated
like this:

``` bash
pnpm generate-entropy
```

That is going to generate a 48 byte long secure random value and store in your
.env file.

## Run

The services can be executed like this:

``` bash
# start key registry updater
pnpm key-registry

# start salt service
pnpm salt-service
```

## Development

For development, they can be executed
in dev mode:

```bash
# start key registry updater
pnpm dev:key-registry

# start salt service
pnpm dev:salt-service
```

To run both in development you can use nx:

``` bash
pnpm nx dev:all
```
