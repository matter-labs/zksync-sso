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

# Build ERC-4337 related packages (following CI build order)
WORKDIR /usr/src/app

# Build in correct dependency order: web-sdk -> sdk-4337 -> auth-server-api
RUN pnpm nx build web-sdk
RUN pnpm nx build sdk-4337 || (echo "===SDK-4337 Build Failed, trying direct build===" && cd packages/sdk-4337 && pnpm build)
RUN pnpm nx build auth-server-api || (echo "===Auth-Server-API Build Failed, trying direct build===" && cd packages/auth-server-api && pnpm build)

# Deploy only production dependencies for auth-server-api
RUN pnpm deploy --filter=auth-server-api --prod /prod/auth-server-api

# Stage 3: Production runtime
FROM node:22-slim AS production

# Install curl for healthcheck
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Copy the deployed auth-server-api with its dependencies
COPY --from=builder /prod/auth-server-api /prod/auth-server-api

# Copy the built dist folder
COPY --from=builder /usr/src/app/packages/auth-server-api/dist /prod/auth-server-api/dist

# Copy sdk-4337 built files to node_modules
COPY --from=builder /usr/src/app/packages/sdk-4337/dist /prod/auth-server-api/node_modules/zksync-sso-4337/dist

# Copy web-sdk built files and WASM packages to .pnpm directory structure
RUN mkdir -p $(find /prod/auth-server-api/node_modules/.pnpm -type d -name "zksync-sso-web-sdk" 2>/dev/null | head -1)
COPY --from=builder /usr/src/app/packages/sdk-platforms/web/pkg-bundler /tmp/pkg-bundler
COPY --from=builder /usr/src/app/packages/sdk-platforms/web/pkg-node /tmp/pkg-node
COPY --from=builder /usr/src/app/packages/sdk-platforms/web/dist /tmp/web-dist
RUN PNPM_WEB_SDK=$(find /prod/auth-server-api/node_modules/.pnpm -type d -name "zksync-sso-web-sdk" 2>/dev/null | head -1) && \
    if [ -n "$PNPM_WEB_SDK" ]; then \
      cp -r /tmp/pkg-bundler "$PNPM_WEB_SDK/" && \
      cp -r /tmp/pkg-node "$PNPM_WEB_SDK/" && \
      cp -r /tmp/web-dist "$PNPM_WEB_SDK/dist"; \
    fi && \
    rm -rf /tmp/pkg-bundler /tmp/pkg-node /tmp/web-dist

# Copy sdk-4337 built files to .pnpm directory structure
COPY --from=builder /usr/src/app/packages/sdk-4337/dist /tmp/sdk-4337-dist
RUN PNPM_SDK_4337=$(find /prod/auth-server-api/node_modules/.pnpm -type d -name "zksync-sso-4337" 2>/dev/null | head -1) && \
    if [ -n "$PNPM_SDK_4337" ]; then \
      cp -r /tmp/sdk-4337-dist "$PNPM_SDK_4337/dist"; \
    fi && \
    rm -rf /tmp/sdk-4337-dist

WORKDIR /prod/auth-server-api

# Environment variables
ENV PORT=3004
ENV NODE_ENV=production

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["curl", "-f", "http://localhost:3004/api/health"]

# Expose port
EXPOSE 3004

# Start the server with WASM support
ENTRYPOINT ["node", "--experimental-wasm-modules", "dist/index.js"]
