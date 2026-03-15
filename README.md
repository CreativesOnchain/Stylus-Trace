# Stylus Trace

[![CI](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml/badge.svg)](https://github.com/CreativesOnchain/Stylus-Trace/actions/workflows/ci.yml)

**A high-performance profiling tool for Arbitrum Stylus transactions.**

Stylus Trace turns opaque Stylus transaction traces into **interactive flamegraphs** and **actionable performance reports**. Profile gas usage, identify bottlenecks, and resolve performance regressions using a local development environment.

---

## 🚀 Key Features

- **Interactive Web Viewer**: Explore transactions in a high-intensity "Cyber Diagnostics" terminal with real-time symbol search and magnitude-sorted deltas.
- **Optimization Insights**: Get qualitative feedback on loop redundancies, high-cost storage access, and potential caching opportunities.
- **Gas & Ink Analysis**: Seamlessly toggle between standard Gas and high-precision Stylus Ink (10,000x) units.
- **Side-by-Side Diffing**: Compare two profiles visually to hunt down regressions or verify optimizations.
- **Automated Artifacts**: Built-in organization for profiles and graphs in a dedicated `artifacts/` folder.
- **Arbitrum Native**: Designed specifically for the Arbitrum Nitro/Stylus execution environment.

---

## 🏗 Project Architecture

Stylus Trace is organized as a Cargo Workspace for modularity and performance:

- `bin/stylus-trace-studio`: The CLI frontend. Optimized for usability and speed.
- `crates/stylus-trace-core`: The core library engine published on [crates.io](https://crates.io/crates/stylus-trace-core).
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

# Profile and immediately launch the Cyber Viewer
stylus-trace capture --tx <TX_HASH> --view

OR

# Generate output as profile.json and interactive flamegraph
stylus-trace capture --tx <TX_HASH> --flamegraph --summary

```

**What happens?**
- `artifacts/profile.json`: A detailed data structure of your transaction.
- `artifacts/viewer.html`: A self-contained, high-performance web viewer.
- `artifacts/flamegraph.svg`: A classic interactive SVG snapshot.
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
| `--tx` | Transaction hash to profile | - |
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
| `--view` | Open the interactive comparison viewer | `false` |

### `view`

| Flag       | Description                                 | Default |
|------------|---------------------------------------------|---------|
| `--target` | **(Required)** Path to profile JSON to view | -       |
| `--baseline` | Optional baseline profile JSON for comparison| -       |

### `ci init`
| Flag | Description | Default |
|------|-------------|---------|
| `--tx` | Transaction hash to profile in CI (optional) | - |
| `--rpc` | RPC endpoint URL | `http://localhost:8547` |
| `--threshold` | Global percentage threshold for all metrics | `1.0` |
| `--gas-threshold` | Specific percentage threshold for Gas regressions | - |
| `--hostio-threshold` | Specific percentage threshold for HostIO regressions | - |
| `--force` | Overwrite existing workflow files | `false` |

---

## 🤖 CI/CD Integration

Stylus Trace is built for automated performance tracking. You can integrate it into your project to **strictly block merges** that cause gas regressions.

### Quick Setup (Zero Config)
Run this in your repository to auto-generate a GitHub Actions workflow:
```bash
stylus-trace ci init
```
*Note: You can optionally provide `--tx 0x...` now or fill it in the generated YAML later.*

### Manual Integration
You can use the [Stylus Trace Action](https://github.com/CreativesOnchain/Stylus-Trace) directly in your workflows:

```yaml
- name: Gas Regression Check
  uses: CreativesOnchain/Stylus-Trace@main
  with:
    tx_hash: "0x..."
    gas_threshold: "1.0" # Fail if gas increases by > 1%
    threshold: "10.0"     # Higher global limit for other metrics
```

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
