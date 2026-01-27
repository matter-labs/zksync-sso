#!/usr/bin/env bash
set -euo pipefail

if [[ ! -d zksync-os-server ]]; then
  echo "Going to clone zksync-os-server"
  git clone https://github.com/matter-labs/zksync-os-server
fi

cd zksync-os-server
echo "Going to build zksync-os-server"
cargo build --release --bin zksync-os-server

echo "listing contents of zksync-os-server directory"
ls 

echo "Going to run anvil"
anvil --load-state zkos-l1-state.json --port 8545 &> anvil.log &

echo "Going to run zksync-os-server"
cargo run --release --bin zksync-os-server &> server.log &
