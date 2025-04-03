#!/bin/bash

DEFAULT_RPC_URL="http://0.0.0.0:8011/"
DEFAULT_RP_ID="https://soo-sdk-example-pages.pages.dev"

VALIDATOR_ADDRESS=""
ACCOUNT_ADDRESS=""
RPC_URL="$DEFAULT_RPC_URL"
RP_ID="$DEFAULT_RP_ID"

print_usage() {
  echo "Usage: $0 --validator <validator_address> --account <account_address> [--rpc-url <rpc_url>] [--rp-id <rp_id>]"
  echo ""
  echo "Options:"
  echo "  --validator    Validator contract address"
  echo "  --account      Account address to check"
  echo "  --rpc-url      RPC URL (default: $DEFAULT_RPC_URL)"
  echo "  --rp-id        Passkey RP ID (default: $DEFAULT_RP_ID)"
  exit 1
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --validator)
      VALIDATOR_ADDRESS="$2"
      shift 2
      ;;
    --account)
      ACCOUNT_ADDRESS="$2"
      shift 2
      ;;
    --rpc-url)
      RPC_URL="$2"
      shift 2
      ;;
    --rp-id)
      RP_ID="$2"
      shift 2
      ;;
    *)
      print_usage
      ;;
  esac
done

if [ -z "$VALIDATOR_ADDRESS" ] || [ -z "$ACCOUNT_ADDRESS" ]; then
  print_usage
fi

check_lower_key_half() {
  local validator_address="$1"
  local rp_id="$2"
  local account_address="$3"
  local rpc_url="$4"
  
  echo "Checking lower key half..."
  cast call "$validator_address" "lowerKeyHalf(string,address)" "$rp_id" "$account_address" --rpc-url "$rpc_url"
}

check_upper_key_half() {
  local validator_address="$1"
  local rp_id="$2"
  local account_address="$3"
  local rpc_url="$4"
  
  echo "Checking upper key half..."
  cast call "$validator_address" "upperKeyHalf(string,address)" "$rp_id" "$account_address" --rpc-url "$rpc_url"
}

echo "Checking public key for account $ACCOUNT_ADDRESS"
echo "Validator address: $VALIDATOR_ADDRESS"
echo "RP ID: $RP_ID"
echo "RPC URL: $RPC_URL"
echo "----------------------------------------"

LOWER_HALF=$(check_lower_key_half "$VALIDATOR_ADDRESS" "$RP_ID" "$ACCOUNT_ADDRESS" "$RPC_URL")
UPPER_HALF=$(check_upper_key_half "$VALIDATOR_ADDRESS" "$RP_ID" "$ACCOUNT_ADDRESS" "$RPC_URL")

echo "----------------------------------------"
echo "Complete Public Key:"
echo "Lower half: $LOWER_HALF"
echo "Upper half: $UPPER_HALF"
echo "----------------------------------------"