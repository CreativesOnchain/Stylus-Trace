# Stylus Trace Studio

**Performance profiling and flamegraph generation for Arbitrum Stylus transactions.**

Stylus Trace Studio turns opaque transaction traces into visual flamegraphs and actionable reports. Profile gas usage, identify bottlenecks, and catch performance regressions—all locally with the Nitro dev node.

---

## Quick Start

### Prerequisites

- **Docker** (for Nitro dev node)
- **Rust** toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Foundry** (for `cast`): `curl -L https://foundry.paradigm.xyz | bash && foundryup`
- **Cargo Stylus**: `cargo install --force cargo-stylus`

### Installation

```bash
git clone https://github.com/Timi16/stylus-trace.git
cd stylus-trace
cargo build --release
```

The binary will be at `./target/release/stylus-trace`.

---

## Complete Testing Guide

### Step 1: Start Nitro Dev Node

```bash
# Clone the Nitro dev node repository
git clone https://github.com/OffchainLabs/nitro-devnode.git
cd nitro-devnode

# Start the dev node (requires Docker)
./run-dev-node.sh
```

**Note:** This script starts a local Arbitrum Nitro node on `http://localhost:8547` with the debug API enabled.

Verify it's running:
```bash
curl -X POST http://localhost:8547 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### Step 2: Deploy a Stylus Contract

```bash
# Create a new Stylus project
cargo stylus new my-contract
cd my-contract

# Example contract (src/lib.rs):
cat > src/lib.rs << 'EOF'
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use stylus_sdk::alloy_primitives::U256;
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 value;
    }
}

#[public]
impl Counter {
    pub fn add(&mut self, left: U256, right: U256) -> U256 {
        let sum = left + right;
        self.value.set(sum);
        sum
    }

    pub fn get_value(&self) -> U256 {
        self.value.get()
    }
}
EOF

# Deploy to local dev node
cargo stylus deploy \
  --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 \
  --endpoint http://localhost:8547
```

**Save the contract address from the output!**

Example output:
```
deployed code at address: 0x525c2aba45f66987217323e8a05ea400c65d06dc
```

### Step 3: Execute a Transaction

```bash
# Set your contract address
CONTRACT_ADDRESS="0x525c2aba45f66987217323e8a05ea400c65d06dc"

# Call the add function
TX_HASH=$(cast send $CONTRACT_ADDRESS \
  "add(uint256,uint256)" 42 58 \
  --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 \
  --rpc-url http://localhost:8547 \
  --json | jq -r '.transactionHash')

echo "Transaction hash: $TX_HASH"

# Wait for confirmation
sleep 2
```

### Step 4: Generate Profile & Flamegraph

```bash
cd /path/to/stylus-trace-studio

./target/release/stylus-trace capture \
  --rpc http://localhost:8547 \
  --tx $TX_HASH \
  --output profile.json \
  --flamegraph flamegraph.svg \
  --summary
```

**Expected output:**
```
[INFO] Starting capture for transaction: 0x51a2d1e5...
[INFO] Step 1/6: Fetching trace from RPC...
[INFO] Step 2/6: Parsing trace data...
[INFO] Step 3/6: Building collapsed stacks...
[INFO] Gas distribution: Total: 22108 gas | Stacks: 6 | Mean: 3684 | Median: 1
[INFO] Step 4/6: Calculating top 20 hot paths...
[INFO] Step 5/6: Generating flamegraph...
[INFO] Flamegraph generated successfully (19047 bytes)
[INFO] Step 6/6: Writing output files...
[INFO] ✓ Profile written to: profile.json
[INFO] ✓ Flamegraph written to: flamegraph.svg

================================================================================
PROFILE SUMMARY
================================================================================
Transaction: 0x51a2d1e5feda3def9455b6ec50a5a77f6b570aa942209432d4ac0bcdc7def4a6
Total Gas:   71136
HostIO Calls: 0
Unique Stacks: 6

Top Gas Consumers:
────────────────────────────────────────────────────────────────────────────────
  1.      22106 gas | call;SSTORE
  2.          1 gas | call;CALLDATACOPY
  3.          1 gas | call;CALLVALUE
  4.          0 gas | call;POP
  5.          0 gas | call;RETURN
  6.          0 gas | call;SLOAD
================================================================================
[INFO] Capture completed in 0.04s
```

### Step 5: View the Flamegraph

Open the SVG in a browser for **interactive features** (hover, zoom, search):

```bash
# macOS
open flamegraph.svg

# Linux
xdg-open flamegraph.svg

# Or specify a browser
open -a "Google Chrome" flamegraph.svg
```

**What you'll see:**
- Hover over boxes to see gas cost and percentage
- Click boxes to zoom in
- Use search (top-right) to highlight specific operations
- Wider boxes = more gas consumed

---

## CLI Command Reference

### Main Commands

```bash
./target/release/stylus-trace --help
```

**Output:**
```
Performance profiling and flamegraph generation for Arbitrum Stylus transactions

Usage: stylus-trace [OPTIONS] <COMMAND>

Commands:
  capture   Capture and profile a transaction
  validate  Validate a profile JSON file
  schema    Display schema information
  version   Display version information
  help      Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Enable verbose logging
  -h, --help     Print help
  -V, --version  Print version
```

### 1. Capture Command

Profile a transaction and generate outputs:

```bash
./target/release/stylus-trace capture \
  --rpc <RPC_URL> \
  --tx <TRANSACTION_HASH> \
  --output <OUTPUT_JSON> \
  --flamegraph <OUTPUT_SVG> \
  --summary
```

**Options:**
- `--rpc <URL>` - RPC endpoint (default: `http://localhost:8547`)
- `--tx <HASH>` - Transaction hash to profile (required)
- `--output <PATH>` - Output path for JSON profile (default: `profile.json`)
- `--flamegraph <PATH>` - Output path for SVG flamegraph (optional)
- `--top-paths <N>` - Number of top hot paths to include (default: 20)
- `--title <TITLE>` - Custom flamegraph title
- `--palette <PALETTE>` - Color scheme: `hot`, `mem`, `io`, `java`, `consistent` (default: `hot`)
- `--width <PIXELS>` - Flamegraph width in pixels (default: 1200)
- `--summary` - Print text summary to stdout

**Examples:**

Basic capture:
```bash
./target/release/stylus-trace capture \
  --rpc http://localhost:8547 \
  --tx 0xabc123... \
  --output my-profile.json \
  --flamegraph my-flamegraph.svg
```

With custom settings:
```bash
./target/release/stylus-trace capture \
  --rpc http://localhost:8547 \
  --tx 0xabc123... \
  --output profile.json \
  --flamegraph flamegraph.svg \
  --title "My Contract Performance" \
  --palette mem \
  --width 1600 \
  --top-paths 50 \
  --summary
```

Verbose logging:
```bash
./target/release/stylus-trace -v capture \
  --rpc http://localhost:8547 \
  --tx 0xabc123... \
  --output profile.json \
  --flamegraph flamegraph.svg
```

### 2. Validate Command

Validate a profile JSON file:

```bash
./target/release/stylus-trace validate --file profile.json
```

**Output:**
```
Validating profile: profile.json
✓ Valid profile JSON
  Version: 1.0.0
  Transaction: 0x51a2d1e5feda3def9455b6ec50a5a77f6b570aa942209432d4ac0bcdc7def4a6
  Total Gas: 71136
  HostIO Calls: 0
  Hot Paths: 6
```

### 3. Schema Command

Display the profile JSON schema:

```bash
./target/release/stylus-trace schema
```

With details:
```bash
./target/release/stylus-trace schema --show
```

### 4. Version Command

Display version information:

```bash
./target/release/stylus-trace version
```

---

## Understanding the Output

### Profile JSON (`profile.json`)

```json
{
  "version": "1.0.0",
  "transaction_hash": "0x51a2d1e5...",
  "total_gas": 71136,
  "hostio_summary": {
    "total_calls": 0,
    "by_type": {},
    "total_hostio_gas": 0
  },
  "hot_paths": [
    {
      "stack": "call;SSTORE",
      "gas": 22106,
      "percentage": 31.07,
      "source_hint": null
    }
  ],
  "generated_at": "2026-01-20T02:41:16.123Z"
}
```

**Fields:**
- `version` - Schema version
- `transaction_hash` - Profiled transaction
- `total_gas` - Total gas used
- `hostio_summary` - HostIO call statistics (Milestone 2)
- `hot_paths` - Top gas-consuming execution paths
- `generated_at` - Profile generation timestamp

### Flamegraph SVG

The flamegraph visualizes gas consumption:
- **X-axis (width)** - Proportion of gas consumed
- **Y-axis (height)** - Call stack depth
- **Colors** - Different operations/modules
- **Interactive** - Hover for details, click to zoom

**Reading the flamegraph:**
1. **Root at bottom** - Entry point of execution
2. **Stacks build up** - Function calls build vertically
3. **Width = gas cost** - Wider boxes consumed more gas
4. **Hot paths** - Widest boxes at each level are optimization targets

---

## Troubleshooting

### Issue: "Connection refused" or RPC errors

**Solution:** Ensure Nitro dev node is running:
```bash
cd nitro-devnode
./run-dev-node.sh
```

Check if port 8547 is accessible:
```bash
curl http://localhost:8547
```

### Issue: "Transaction not found"

**Solution:** 
- Wait a few seconds after sending the transaction
- Verify the transaction hash is correct
- Check the transaction was mined:
  ```bash
  cast receipt $TX_HASH --rpc-url http://localhost:8547
  ```

### Issue: "Empty stack data" or "No stack counts found"

**Causes:**
1. **Wrong transaction type** - Deployment/activation transactions have no execution trace
2. **No gas costs** - Some operations may have 0 gas cost

**Solution:**
- Only profile **function call transactions**, not deployments
- Ensure you're calling an actual contract function:
  ```bash
  # Good - function call
  cast send $CONTRACT "add(uint256,uint256)" 42 58 ...
  
  # ❌ Bad - deployment transaction
  cargo stylus deploy ...
  ```

### Issue: Flamegraph looks too simple

**Explanation:** Simple contracts produce simple flamegraphs. A contract that only does one storage write will show one dominant operation.

**Solution:** Test with more complex contracts that:
- Make multiple storage operations
- Call other contracts
- Perform loops or complex computations

### Issue: "Invalid hex" or "Transaction hash must be 32 bytes"

**Solution:** Ensure the transaction hash:
- Is 64 hex characters (32 bytes)
- Includes `0x` prefix (or is exactly 64 chars without it)
- Contains only valid hex characters (0-9, a-f)

---

## Best Practices

### 1. Profile Function Calls, Not Deployments

```bash
# Profile this
TX=$(cast send $CONTRACT "myFunction()" ... --json | jq -r '.transactionHash')
./target/release/stylus-trace capture --tx $TX ...

# ❌ Don't profile this
DEPLOY_TX=$(cargo stylus deploy ... | grep "deployment tx hash")
./target/release/stylus-trace capture --tx $DEPLOY_TX ...  # Will fail!
```

### 2. Use Meaningful Titles

```bash
./target/release/stylus-trace capture \
  --tx $TX \
  --title "TokenSwap.swap() - Before Optimization" \
  --output before.json \
  --flamegraph before.svg
```

### 3. Compare Before/After (Milestone 2 Preview)

```bash
# Profile original version
./target/release/stylus-trace capture --tx $TX1 --output before.json --flamegraph before.svg

# Make changes, redeploy, run same transaction
./target/release/stylus-trace capture --tx $TX2 --output after.json --flamegraph after.svg

# Compare visually by opening both flamegraphs
open before.svg after.svg
```

### 4. Save Profiles for CI/Review

```bash
# Organize by git commit
mkdir -p profiles/$(git rev-parse --short HEAD)
./target/release/stylus-trace capture \
  --tx $TX \
  --output profiles/$(git rev-parse --short HEAD)/profile.json \
  --flamegraph profiles/$(git rev-parse --short HEAD)/flamegraph.svg
```

---

## Example Workflows

### Basic Profiling

```bash
#!/bin/bash
set -e

# 1. Deploy contract
cd my-stylus-contract
CONTRACT=$(cargo stylus deploy \
  --private-key $PRIVATE_KEY \
  --endpoint http://localhost:8547 | \
  grep "deployed code at address:" | \
  awk '{print $NF}')

# 2. Execute transaction
TX=$(cast send $CONTRACT \
  "myFunction(uint256)" 42 \
  --private-key $PRIVATE_KEY \
  --rpc-url http://localhost:8547 \
  --json | jq -r '.transactionHash')

# 3. Profile
cd ../stylus-trace-studio
./target/release/stylus-trace capture \
  --rpc http://localhost:8547 \
  --tx $TX \
  --output profile.json \
  --flamegraph flamegraph.svg \
  --summary

# 4. View
open flamegraph.svg
```

### Batch Profiling

```bash
#!/bin/bash
# Profile multiple transactions

TRANSACTIONS=(
  "0xabc123..."
  "0xdef456..."
  "0x789abc..."
)

for i in "${!TRANSACTIONS[@]}"; do
  ./target/release/stylus-trace capture \
    --rpc http://localhost:8547 \
    --tx "${TRANSACTIONS[$i]}" \
    --output "profile-$i.json" \
    --flamegraph "flamegraph-$i.svg" \
    --title "Transaction $i"
done
```

---

## Additional Resources

- [Arbitrum Stylus Documentation](https://docs.arbitrum.io/stylus/stylus-gentle-introduction)
- [Nitro Dev Node](https://github.com/OffchainLabs/nitro-devnode)
- [Cargo Stylus](https://github.com/OffchainLabs/cargo-stylus)
- [Flamegraph Visualization](https://www.brendangregg.com/flamegraphs.html)

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

[MIT License](LICENSE)

---

## Support

- **Issues**: [GitHub Issues](https://github.com/Timi16/stylus-trace/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Timi16/stylus-trace/discussions)


---

**Built with ❤️ for the Arbitrum ecosystem**