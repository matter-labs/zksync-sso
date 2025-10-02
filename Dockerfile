
FROM node:22-slim AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable pnpm

FROM base AS build
COPY . /usr/src/app
WORKDIR /usr/src/app
RUN pnpm install --prod=false --frozen-lockfile
# Build packages in dependency order: circuits first, then contracts if needed, then SDK, then oidc-server
RUN pnpm --filter=zksync-sso-circuits run build
RUN pnpm --filter=zksync-sso-contracts run build
RUN pnpm --filter=sdk run build
RUN pnpm --filter=zksync-sso-oidc-server run build
# Skip web SDK and other Rust packages that require wasm-pack/cargo
RUN pnpm deploy --filter=zksync-sso-oidc-server --prod /prod/oidc-server

FROM base AS oidc-server
COPY --from=build /prod/oidc-server /prod/oidc-server
WORKDIR /prod/oidc-server
COPY --from=build /usr/src/app/packages/oidc-server/dist ./dist
ENV SALT_SERVICE_PORT=3003
# Expose main service port and Prometheus metrics port
EXPOSE 3003 9090
CMD [ "node", "dist/salt-service.js" ]

FROM oidc-server AS key-registry
CMD [ "node", "dist/update-keys-service.js" ]
