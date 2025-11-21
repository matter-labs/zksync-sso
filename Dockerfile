
FROM node:22-slim AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable pnpm

FROM base AS build
COPY . /usr/src/app
WORKDIR /usr/src/app
RUN pnpm install --prod=false --frozen-lockfile
RUN pnpm --filter='!./packages/sdk-platforms/*' --filter='!./packages/sdk-4337' --filter='!./packages/auth-server-api' -r run build
RUN pnpm deploy --filter=oidc-server --prod /prod/oidc-server

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