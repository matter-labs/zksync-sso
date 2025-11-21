# Bundler

ERC-4337 bundler service with CORS proxy for ZKsync SSO. Built on
[@pimlico/alto](https://github.com/pimlicolabs/alto).

## Features

- **Alto Bundler**: ERC-4337 compliant bundler on port 4338
- **CORS Proxy**: Browser-friendly proxy on port 4337
- **Environment Validation**: Zod-based config with sensible defaults
- **Docker Ready**: Multi-stage build with production-optimized image

## Ports

- **4337**: CORS proxy (accessible from browsers)
- **4338**: Alto bundler API (internal)

## Quick Start

### Local Development

Works out of the box with local Anvil:

```bash
# Install dependencies
pnpm install

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
RPC_URL=https://sepolia.drpc.org
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
docker run -p 4337:4337 -p 4338:4338 \
  -e EXECUTOR_PRIVATE_KEY=0x... \
  -e UTILITY_PRIVATE_KEY=0x... \
  -e RPC_URL=https://sepolia.drpc.org \
  sso-bundler
```

### Pre-built Image

```bash
docker pull matterlabs/sso-bundler:latest

docker run -p 4337:4337 -p 4338:4338 \
  -e EXECUTOR_PRIVATE_KEY=0x... \
  -e UTILITY_PRIVATE_KEY=0x... \
  -e RPC_URL=https://sepolia.drpc.org \
  matterlabs/sso-bundler:latest
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

## API Usage

Send ERC-4337 user operations to: `http://localhost:4337`

The CORS proxy automatically forwards requests to Alto bundler and adds
appropriate CORS headers for browser compatibility.

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
