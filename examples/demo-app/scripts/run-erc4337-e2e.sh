#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

AUTH_API_PID=""
AUTH_FRONTEND_PID=""
DEMO_PID=""

cleanup() {
  for pid in "$DEMO_PID" "$AUTH_FRONTEND_PID" "$AUTH_API_PID"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
}

wait_for_http() {
  local url="$1"
  local label="$2"
  local attempts="${3:-90}"

  for ((i = 1; i <= attempts; i++)); do
    if curl -sSf "$url" >/dev/null 2>&1 || curl -sI "$url" >/dev/null 2>&1; then
      echo "$label is ready at $url"
      return 0
    fi
    sleep 1
  done

  echo "Timed out waiting for $label at $url"
  return 1
}

trap cleanup EXIT INT TERM

cd "$WORKSPACE_ROOT"

echo "Building auth-server frontend..."
pnpm nx run auth-server:build:local >/tmp/auth-server-build.log 2>&1

echo "Building demo-app preview..."
pnpm nx run demo-app:build:local >/tmp/demo-app-build.log 2>&1

echo "Starting auth-server API..."
RPC_URL=http://127.0.0.1:3050 PORT=3004 pnpm nx run auth-server-api:dev >/tmp/auth-server-api.log 2>&1 &
AUTH_API_PID=$!
wait_for_http "http://127.0.0.1:3004" "auth-server API"

echo "Starting auth-server preview..."
NUXT_PUBLIC_AUTH_SERVER_API_URL=http://localhost:3004 NUXT_PUBLIC_CHAIN_RPC_URL=http://localhost:3050 PORT=3002 pnpm -C packages/auth-server exec nuxt preview >/tmp/auth-server-preview.log 2>&1 &
AUTH_FRONTEND_PID=$!
wait_for_http "http://127.0.0.1:3002" "auth-server preview"

echo "Starting demo-app preview..."
PORT=3005 pnpm -C examples/demo-app exec nuxt preview >/tmp/demo-app-preview.log 2>&1 &
DEMO_PID=$!
wait_for_http "http://127.0.0.1:3005" "demo-app preview"

echo "Running Playwright ERC-4337 suite..."
PW_MANAGED_SERVERS=1 PW_TEST_HTML_REPORT_OPEN=never pnpm --dir examples/demo-app exec playwright test --config=playwright-erc4337.config.ts "$@"
