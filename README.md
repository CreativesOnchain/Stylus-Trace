# Stylus Trace

[![CI](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml/badge.svg)](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml)

**A high-performance profiling tool for Arbitrum Stylus transactions.**

Stylus Trace turns opaque Stylus transaction traces into **interactive flamegraphs** and **actionable performance reports**. Profile gas usage, identify bottlenecks, and resolve performance regressions using a local development environment.

---

## 🚀 Key Features

- **Interactive Flamegraphs**: Visualize execution paths with interactive SVG snapshots.
- **Gas & Ink Analysis**: Seamlessly toggle between standard Gas and high-precision Stylus Ink (10,000x) units.
- **Transaction Dashboards**: Get a "hot path" summary directly in your terminal.
- **Automated Artifacts**: Built-in organization for profiles and graphs in a dedicated `artifacts/` folder.
- **Arbitrum Native**: Designed specifically for the Arbitrum Nitro/Stylus execution environment.

---

## 🏗 Project Architecture

Stylus Trace is organized as a Cargo Workspace for modularity and performance:

- `bin/stylus-trace`: The CLI frontend. Optimized for usability and speed.
- `crates/stylus-trace-studio`: The core library engine published on [crates.io](https://crates.io/crates/stylus-trace-studio).
- `artifacts/`: Standardized output directory for profiles and flamegraphs (Git ignored).

---

## 📦 Installation

### Via Cargo (Recommended)
You can install the CLI directly from crates.io:
```bash
cargo install stylus-trace-studio
```

### Build from Source (Host Native)
If you prefer to build from the latest source code on your machine:
```bash
# Clone the repository
git clone https://github.com/CreativesOnchain/Stylus-Trace.git
cd Stylus-Trace

# Install from the workspace (Native build, NOT WASM)
cargo install --path bin/stylus-trace
```

---

## 🛠 Quick Start

### 1. Prerequisites
Ensure you have the following installed:
- **Docker** (for Nitro dev node)
- **Rust** (1.72+)
- **Foundry** (`cast`)
- **Cargo Stylus** (`cargo install --force cargo-stylus`)

### Step 1: Start Nitro Dev Node

```bash
git clone https://github.com/OffchainLabs/nitro-devnode.git
cd nitro-devnode && ./run-dev-node.sh
```

### 3. Build & Profile your Contract
In your **contract's** directory:

```bash
# Build the contract to WASM
cargo build --release --target wasm32-unknown-unknown

# Capture profile fully
stylus-trace capture \
--rpc <RPC> \
--tx <TX_HASH> \
--output <anything.json> \
--flamegraph <anything.svg> \
--summary

OR

# Capture profile with default options
stylus-trace capture --tx <TX_HASH> --summary

OR

stylus-trace capture \
--tx <TX_HASH> \
--output <anything.json> \
--flamegraph <anything.svg> \
--summary

```

**What happens?**
- `artifacts/profile.json`: A detailed data structure of your transaction.
- `artifacts/flamegraph.svg`: An interactive SVG you can open in any browser.
- **Terminal Output**: A high-level summary of the hottest paths.

---

## Source-to-Line Mapping

To enable line-level resolution in your reports and flamegraphs, you must compile your Stylus contract with **DWARF debug symbols**.

### 1. Enable Debug Info
Update your contract's `Cargo.toml`:

   ```toml
   [profile.release]
   debug = true
   ```

2. **Build your contract**:
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

---

## 📖 CLI Command Reference

### `capture`
| Flag | Description | Default |
|------|-------------|---------|
| `--tx` | **(Required)** Transaction hash to profile | - |
| `--rpc` | RPC endpoint URL | `http://localhost:8547` |
| `--flamegraph` | Generate an SVG flamegraph | `artifacts/flamegraph.svg` |
| `--output` | Save JSON profile to path | `artifacts/profile.json` |
| `--summary` | Print a text-based summary to terminal | `false` |
| `--ink` | Use Stylus Ink units (scaled 10,000x) | `false` |
| `--wasm` | Path to WASM binary for source mapping | - |

---

## 🤝 Contributing

We welcome contributions! 

```bash
# Run tests across workspace
cargo test --workspace

# Linting
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Formatting
cargo fmt --all --check
```

---

## 📄 License

MIT

**Built with ❤️ for the Arbitrum Stylus ecosystem.**
