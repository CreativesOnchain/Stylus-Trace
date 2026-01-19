#!/bin/bash
set -e

echo "🧪 Testing Milestone 1..."

# 1. Send transaction
echo "Sending test transaction..."
TX_HASH=$(cast send 0x0000000000000000000000000000000000000001 \
  --value 0.1ether \
  --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 \
  --rpc-url http://localhost:8547 \
  --json | jq -r '.transactionHash')

echo "✅ TX: $TX_HASH"

# 2. Wait for confirmation
sleep 2

# 3. Run our tool
echo "Running stylus-trace..."
./target/release/stylus-trace capture \
  --rpc http://localhost:8547 \
  --tx $TX_HASH \
  --output test-profile.json \
  --flamegraph test-flamegraph.svg \
  --summary

echo "✅ Done!"
ls -lh test-profile.json test-flamegraph.svg