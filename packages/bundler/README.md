# Bundler

ERC-4337 bundler service with CORS proxy for ZKsync SSO. Built on
[@pimlico/alto](https://github.com/pimlicolabs/alto).

## Features

- **Alto Bundler**: ERC-4337 compliant bundler on port 4338
- **CORS Proxy**: Browser-friendly proxy on port 4337
- **Environment Validation**: Zod-based config with sensible defaults
- **Docker Ready**: Multi-stage build with production-optimized image

## Ports

- **4337**: Bundler CORS proxy (browser -> Alto)
- **4338**: Alto bundler API (internal)
- **4339**: RPC CORS proxy (browser -> zkSync OS RPC)

## Quick Start

### Local Development

Works out of the box with local Anvil:

```bash
# Install dependencies
pnpm install

# Setup private keys
Create `.env` file similar to .env.example
Update private keys in alto-config.json

# Start bundler (uses default anvil keys)
pnpm dev
```

The bundler will start with default configuration for local development:

- Anvil rich account keys
- RPC: `http://localhost:8545`
- EntryPoint: `0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108`

### Production

Create a `.env` file:

```bash
EXECUTOR_PRIVATE_KEY=0x...
UTILITY_PRIVATE_KEY=0x...
RPC_URL=https://zksync-os-testnet-alpha.zksync.dev/
```

Then start:

```bash
pnpm start
```

## Docker Usage

### Build

```bash
docker build -t sso-bundler packages/bundler
```

### Run

```bash
docker run -p 4337:4337 -p 4338:4338 -p 4339:4339 \
  -e EXECUTOR_PRIVATE_KEY=0x... \
  -e UTILITY_PRIVATE_KEY=0x... \
  -e RPC_URL=https://zksync-os-testnet-alpha.zksync.dev/ \
  sso-bundler
```

### Pre-built Image

```bash
docker pull ghcr.io/matter-labs/sso-bundler:latest

docker run -p 4337:4337 -p 4338:4338 -p 4339:4339 \
  -e EXECUTOR_PRIVATE_KEY=0x... \
  -e UTILITY_PRIVATE_KEY=0x... \
  -e RPC_URL=https://zksync-os-testnet-alpha.zksync.dev/ \
  ghcr.io/matter-labs/sso-bundler:latest
```

## Environment Variables

| Variable               | Description                               | Default (Local Dev)     |
| ---------------------- | ----------------------------------------- | ----------------------- |
| `EXECUTOR_PRIVATE_KEY` | Private key for executing user operations | Anvil account #0        |
| `UTILITY_PRIVATE_KEY`  | Private key for utility operations        | Anvil account #1        |
| `RPC_URL`              | RPC endpoint for blockchain network       | `http://localhost:8545` |

## Configuration

The bundler generates an Alto config file at runtime with these fixed values:

- **EntryPoint**: `0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108`
- **Port**: 4338
- **Safe Mode**: false
- **RPC Proxy**: forwards to `RPC_URL` on port 4339

## API Usage

Send ERC-4337 user operations to: `http://localhost:4337`

Send standard JSON-RPC calls (e.g. `eth_getBalance`) to: `http://localhost:4339`

Both proxies automatically forward requests and add permissive CORS headers for
browser compatibility.

## Development

```bash
# Install dependencies
pnpm install

# Build TypeScript
pnpm build

# Run in development mode with hot reload
pnpm dev

# Run compiled version
pnpm start
```

## License

MIT
