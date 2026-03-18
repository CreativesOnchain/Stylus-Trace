# Stylus Trace Starter Repository

This starter includes:
- A Stylus contract (`contracts/stylus-counter`)
- A Solidity caller (`contracts/solidity/src/CounterCaller.sol`)
- A CI workflow that generates and uploads flamegraph artifacts on first run

## Quickstart (target: under 10 minutes)

1. Clone this starter and enter the directory.
2. Install Rust and Foundry.
3. Install the tracer CLI:
   ```bash
   cargo install stylus-trace-studio
   ```
4. Run a local profile check:
   ```bash
   ./scripts/run-local-profile.sh
   ```
5. Open artifacts:
   - `artifacts/diff/diff.svg`
   - `artifacts/diff/diff_report.json`

## CI behavior

The included workflow always creates artifacts from sample profiles, so it is green on first run.

Optional live capture is supported by setting repository variables:
- `STYLUS_TRACE_RPC_URL`
- `STYLUS_TRACE_TX_HASH`

When both are set, CI captures a live profile before running the diff.

## Contracts

- Stylus contract: `contracts/stylus-counter/src/lib.rs`
- Solidity caller: `contracts/solidity/src/CounterCaller.sol`
