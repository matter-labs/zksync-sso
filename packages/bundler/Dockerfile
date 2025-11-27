# Stage 1: Build TypeScript
FROM node:22-slim AS builder

WORKDIR /app

# Copy package files
COPY package.json ./
COPY tsconfig.json ./

# Install dependencies
RUN npm install

# Copy source files
COPY src/ ./src/

# Build TypeScript
RUN npm run build

# Stage 2: Production runtime
FROM node:22-slim AS production

# Install curl for healthcheck
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install @pimlico/alto globally
RUN npm install -g @pimlico/alto@0.0.19

# Ensure npm global bin is in PATH
ENV PATH="/usr/local/bin:$PATH"

WORKDIR /app

# Copy package.json for production dependencies
COPY package.json ./

# Install production dependencies only
RUN npm install --omit=dev && npm cache clean --force

# Copy built files from builder
COPY --from=builder /app/dist ./dist

# Environment defaults (can be overridden at runtime)
ENV EXECUTOR_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
ENV UTILITY_PRIVATE_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
ENV RPC_URL="http://localhost:8545"

# Healthcheck on CORS proxy port (external)
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s --retries=3 \
  CMD ["curl", "-f", "http://localhost:4337/health"]

# Expose both the CORS proxy port and Alto bundler port
EXPOSE 4337 4338

# Start the bundler
CMD ["node", "dist/index.js"]
