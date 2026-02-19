# Stylus Trace

[![CI](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml/badge.svg)](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml)

**A high-performance profiling tool for Arbitrum Stylus transactions.**

Stylus Trace turns opaque Stylus transaction traces into **interactive flamegraphs** and **actionable performance reports**. Profile gas usage, identify bottlenecks, and resolve performance regressions using a local development environment.

---

## 🚀 Key Features

- **Interactive Flamegraphs**: Visualize execution paths with interactive SVG snapshots.
- **Optimization Insights**: Get qualitative feedback on loop redundancies, high-cost storage access, and potential caching opportunities.
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

# This will generate output as profile.json and flamegraph.svg in specified file
stylus-trace capture \
--rpc <RPC> \
--tx <TX_HASH> \
--output <anything.json> \
--flamegraph <anything.svg> \
--summary

OR

# This will generate output as profile.json but no flamegraph in artifacts directory
stylus-trace capture --tx <TX_HASH> --summary

OR
# this will generate output as profile.json and flamegraph.svg in artifacts directory
stylus-trace capture \
--tx <TX_HASH> \
--flamegraph \
--summary

```

**What happens?**
- `artifacts/profile.json`: A detailed data structure of your transaction.
- `artifacts/flamegraph.svg`: An interactive SVG you can open in any browser.
- **Terminal Output**: A high-level summary of the hottest paths.

---

## 📖 Source-to-Line Mapping (Reserved Feature)

Line-level resolution in reports and flamegraphs is a **reserved feature**. While the engine supports DWARF debug symbols, it is currently **non-functional** because the Arbitrum `stylusTracer` does not yet provide the required Program Counter (PC) offsets for WASM execution.

This feature will be enabled automatically once upstream tracer support is available.

---

## 📖 CLI Command Reference

### `capture`
| Flag | Description | Default |
|------|-------------|---------|
| `--tx` | **(Required)** Transaction hash to profile | - |
| `--rpc` | RPC endpoint URL | `http://localhost:8547` |
| `--flamegraph` | Generate an SVG flamegraph | `artifacts/capture/flamegraph.svg` |
| `--output` | Save JSON profile to path | `artifacts/capture/profile.json` |
| `--summary` | Print a text-based summary to terminal | `false` |
| `--ink` | Use Stylus Ink units (scaled 10,000x) | `false` |
| `--tracer` | Optional tracer name | `stylusTracer` |
| `--baseline` | Path to baseline profile for on-the-fly diffing | - |
| `--threshold-percent` | Simple percentage tolerance for **all** metrics (Gas, HostIOs, Hot Paths) | - |
| `--gas-threshold` | Specific percentage tolerance for Gas regressions only | - |
| `--hostio-threshold` | Specific percentage tolerance for total HostIO calls only | - |

### `diff`
| Flag | Description | Default |
|------|-------------|---------|
| `<BASELINE>` | **(Required)** Path to baseline profile JSON | - |
| `<TARGET>` | **(Required)** Path to target profile JSON | - |
| `--threshold-percent` | Simple percentage tolerance for **all** metrics (Gas, HostIOs, Hot Paths) | - |
| `--gas-threshold` | Focus strictly on Gas regressions (overrides TOML/defaults) | - |
| `--hostio-threshold` | Focus strictly on HostIO regressions (overrides TOML/defaults) | - |
| `--threshold` | Optional threshold config file (TOML) | `thresholds.toml` (auto-loaded if exists) |
| `--summary` | Print human-readable summary to terminal | `true` |
| `--output` | Path to write the diff report JSON | `artifacts/diff/diff_report.json` |
| `--flamegraph` | Path to write visual diff flamegraph SVG | `artifacts/diff/diff.svg` |

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
