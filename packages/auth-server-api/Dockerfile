# Stage 1: Build dependencies (Rust + Foundry)
FROM rustlang/rust:nightly-bookworm AS rust-builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Add wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install Foundry (forge, cast, anvil)
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:${PATH}"
RUN foundryup

# Stage 2: Build Node.js dependencies and contracts
FROM node:22-bookworm-slim AS builder

# Copy Foundry and Rust tools from previous stage
COPY --from=rust-builder /usr/local/cargo /usr/local/cargo
COPY --from=rust-builder /usr/local/rustup /usr/local/rustup
COPY --from=rust-builder /root/.foundry /root/.foundry
ENV PATH="/root/.foundry/bin:/usr/local/cargo/bin:${PATH}"
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo

# Enable pnpm
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable pnpm

# Install build dependencies for native modules
RUN apt-get update && apt-get install -y \
    git \
    python3 \
    make \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/app

# Copy the entire monorepo
COPY . .

# Install all dependencies
RUN pnpm install -r --frozen-lockfile

# Build ERC-4337 contracts with Foundry
WORKDIR /usr/src/app/packages/erc4337-contracts
RUN forge soldeer install
RUN forge build

# Build ERC-4337 related packages
WORKDIR /usr/src/app
RUN pnpm nx build web-sdk
RUN pnpm nx build sdk-4337

# Build auth-server-api
RUN pnpm nx build auth-server-api

# Deploy only production dependencies for auth-server-api
RUN pnpm deploy --filter=auth-server-api --prod /prod/auth-server-api

# Stage 3: Production runtime
# Using distroless for minimal attack surface
FROM gcr.io/distroless/nodejs22-debian12:nonroot AS production

# Copy the deployed auth-server-api with its dependencies
COPY --from=builder --chown=nonroot:nonroot /prod/auth-server-api /prod/auth-server-api

# Copy the built dist folder
COPY --from=builder --chown=nonroot:nonroot /usr/src/app/packages/auth-server-api/dist /prod/auth-server-api/dist

WORKDIR /prod/auth-server-api

# Environment variables
ENV PORT=3004
ENV NODE_ENV=production

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["node", "-e", "require('http').get('http://localhost:3004/api/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"]

# Expose port
EXPOSE 3004

# Start the server with WASM support
ENTRYPOINT ["node", "--experimental-wasm-modules", "dist/index.js"]
