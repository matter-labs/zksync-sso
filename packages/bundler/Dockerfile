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

# Install @pimlico/alto globally
RUN npm install -g @pimlico/alto@0.0.19

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

# Healthcheck on Alto bundler port (internal)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["node", "-e", "require('http').get('http://localhost:4338/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"]

# Expose only the CORS proxy port (4338 is internal)
EXPOSE 4337

# Start the bundler
CMD ["node", "dist/index.js"]
