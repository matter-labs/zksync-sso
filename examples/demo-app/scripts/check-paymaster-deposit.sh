#!/bin/bash
set -e

# Configuration
RPC_URL="http://localhost:8545"
ENTRYPOINT="0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108"

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Read paymaster address from contracts-anvil.json
CONTRACTS_FILE="$WORKSPACE_ROOT/examples/demo-app/contracts-anvil.json"

if [ ! -f "$CONTRACTS_FILE" ]; then
  echo "❌ contracts-anvil.json not found at $CONTRACTS_FILE"
  echo "Please run deploy-msa-anvil.sh first"
  exit 1
fi

PAYMASTER=$(jq -r '.testPaymaster // .mockPaymaster' "$CONTRACTS_FILE")

if [ -z "$PAYMASTER" ] || [ "$PAYMASTER" == "null" ]; then
  echo "❌ Paymaster address not found in contracts-anvil.json"
  exit 1
fi

echo "🔍 Checking paymaster deposit status..."
echo ""
echo "Paymaster address: $PAYMASTER"
echo "EntryPoint address: $ENTRYPOINT"
echo ""

# Check paymaster's ETH balance
echo "💰 Paymaster contract balance:"
BALANCE=$(cast balance "$PAYMASTER" --rpc-url "$RPC_URL")
echo "  $BALANCE wei ($(cast --to-unit "$BALANCE" ether) ETH)"
echo ""

# Check deposit in EntryPoint using balanceOf(address)
echo "💳 Deposit in EntryPoint:"
DEPOSIT_RAW=$(cast call "$ENTRYPOINT" "balanceOf(address)(uint256)" "$PAYMASTER" --rpc-url "$RPC_URL")
# Extract just the number without the scientific notation
DEPOSIT=$(echo "$DEPOSIT_RAW" | awk '{print $1}')
DEPOSIT_ETH=$(cast --to-unit "$DEPOSIT" ether 2>/dev/null || echo "0")
echo "  $DEPOSIT wei ($DEPOSIT_ETH ETH)"
echo ""

# Convert to decimal for comparison
DEPOSIT_DEC="$DEPOSIT"

if [ "$DEPOSIT_DEC" == "0" ]; then
  echo "❌ Paymaster has NO deposit in EntryPoint!"
  echo ""
  echo "To fix this, run:"
  echo "  cast send '$PAYMASTER' 'deposit()' --value 10ether --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --rpc-url '$RPC_URL'"
  exit 1
else
  echo "✅ Paymaster has deposit in EntryPoint"
fi

echo "✅ Deposit amount: $DEPOSIT_ETH ETH"
echo ""

# Check stake using getDepositInfo(address)
echo "🔒 Checking stake status:"
DEPOSIT_INFO=$(cast call "$ENTRYPOINT" "getDepositInfo(address)" "$PAYMASTER" --rpc-url "$RPC_URL")

# Parse the returned tuple: (deposit, staked, stake, unstakeDelaySec, withdrawTime)
# The output is a hex string with 5 values concatenated
STAKED_HEX=$(echo "$DEPOSIT_INFO" | cut -c67-130)  # Second 32 bytes (staked boolean)
STAKE_HEX=$(echo "$DEPOSIT_INFO" | cut -c131-194) # Third 32 bytes (stake amount)

# Convert hex to decimal (remove leading zeros)
STAKED_VALUE="0x$STAKED_HEX"
STAKE_VALUE="0x$STAKE_HEX"

# Check if staked (any non-zero value means true)
if [ "$STAKED_VALUE" != "0x0000000000000000000000000000000000000000000000000000000000000000" ]; then
  STAKE_ETH=$(cast --to-unit "$STAKE_VALUE" ether 2>/dev/null || echo "0")
  echo "  ✅ Paymaster is staked"
  echo "  ✅ Stake amount: $STAKE_ETH ETH"
else
  echo "  ❌ Paymaster is NOT staked!"
  echo ""
  echo "  To fix this, run:"
  echo "  cast send '$PAYMASTER' 'addStake(uint32)' 86400 --value 1ether --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --rpc-url '$RPC_URL'"
  exit 1
fi
