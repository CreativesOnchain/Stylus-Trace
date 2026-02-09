# Stylus Trace

[![CI](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml/badge.svg)](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml)

**Performance profiling and flamegraph generation for Arbitrum Stylus transactions.**

Stylus Trace turns opaque Stylus transaction traces into **interactive flamegraphs** and **actionable performance reports**.  
Profile gas usage, identify bottlenecks, and catch regressions — all locally using the Arbitrum Nitro dev node.

Built for the **Arbitrum Stylus ecosystem**.

---

## Quick Start

### Prerequisites

- **Docker** (for Nitro dev node)
- **Rust** (1.72+)
- **Foundry** (`cast`)
- **Cargo Stylus**

```bash
cargo install --force cargo-stylus
```

---

### Installation

```bash
cargo install stylus-trace
```

Verify:
```bash
stylus-trace help
```

---

## Complete Testing Guide

### Step 1: Start Nitro Dev Node

```bash
git clone https://github.com/OffchainLabs/nitro-devnode.git
cd nitro-devnode
./run-dev-node.sh
```

This starts a local Arbitrum node at:
```
http://localhost:8547
```

Verify:
```bash
curl -X POST http://localhost:8547 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

---

### Step 2: Deploy a Stylus Contract

```bash
cargo stylus new my-contract
cd my-contract
```

Example contract (`src/lib.rs`):

```rust
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
```

Deploy:
```bash
cargo stylus deploy \
  --private-key <PRIVATE_KEY> \
  --endpoint http://localhost:8547
```

---

### Step 3: Execute a Transaction

```bash
CONTRACT_ADDRESS="0x..."

TX_HASH=$(cast send $CONTRACT_ADDRESS \
  "add(uint256,uint256)" 42 58 \
  --private-key <PRIVATE_KEY> \
  --rpc-url http://localhost:8547 \
  --json | jq -r '.transactionHash')
```

---

### Step 4: Generate Profile & Flamegraph

```bash
stylus-trace capture \
  --rpc http://localhost:8547 \
  --tx $TX_HASH \
  --wasm ./target/wasm32-unknown-unknown/release/my_contract.wasm \
  --output profile.json \
  --flamegraph flamegraph.svg \
  --summary
```

---

### Step 5: View the Flamegraph

```bash
open flamegraph.svg   # macOS
xdg-open flamegraph.svg  # Linux
```

---

## Source-to-Line Mapping

To enable line-level resolution in your reports and flamegraphs, you must compile your Stylus contract with **DWARF debug symbols**.

### 1. Enable Debug Info
Update your contract's `Cargo.toml`:

```toml
[profile.release]
debug = true
```

### 2. Build Contract
```bash
cargo build --release --target wasm32-unknown-unknown
```

### 3. Capture with WASM
Provide the path to the compiled `.wasm` file using the `--wasm` flag:

```bash
stylus-trace capture --tx <TX_HASH> --wasm path/to/contract.wasm --summary
```

---

## CLI Command Reference

```bash
stylus-trace --help
```

Commands:
- `capture`: Profile a transaction and generate reports.
    - `--rpc`: RPC endpoint URL.
    - `--tx`: Transaction hash to profile.
    - `--wasm`: Path to WASM binary (for source mapping).
    - `--tracer`: Optional tracer name (e.g., `stylusTracer`).
    - `--output`: Path to save JSON profile.
    - `--flamegraph`: Path to save SVG flamegraph.
    - `--ink`: Display costs in Stylus Ink (scaled by 10,000).
- `validate`: Validate a profile JSON file against the schema.
- `schema`: Display the JSON schema for profiles.
- `version`: Display version information.

---

## License

MIT

---

**Built with ❤️ for the Arbitrum Stylus ecosystem**
