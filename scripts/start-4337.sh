#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_GZ="$ROOT_DIR/dev/zksyncos/l1-state.json.gz"
STATE_JSON="$ROOT_DIR/dev/zksyncos/l1-state.json"
COMPOSE_FILE="$ROOT_DIR/docker-compose.4337.yml"

if [ ! -f "$STATE_GZ" ]; then
  echo "Missing compressed L1 state snapshot: $STATE_GZ" >&2
  exit 1
fi

if [ ! -f "$STATE_JSON" ]; then
  echo "Decompressing L1 state snapshot..."
  gunzip -c "$STATE_GZ" > "$STATE_JSON"
fi

echo "Starting local zksync-os stack..."
docker compose -f "$COMPOSE_FILE" up -d l1 zksyncos

echo "Waiting for zksync-os RPC on http://127.0.0.1:3050 ..."
for _ in $(seq 1 90); do
  if curl -fsS \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    http://127.0.0.1:3050 >/dev/null 2>&1; then
    echo "zksync-os is ready."
    echo "Predeploying bundler prerequisites..."
    pnpm --dir "$ROOT_DIR/packages/erc4337-contracts" run bundler:prepare
    exit 0
  fi
  sleep 2
done

echo "zksync-os did not become ready in time." >&2
docker compose -f "$COMPOSE_FILE" logs --tail=200 l1 zksyncos || true
exit 1
