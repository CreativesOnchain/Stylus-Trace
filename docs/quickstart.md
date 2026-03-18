# Quickstart

Target KPI: first profile in about 10 minutes.

## Prerequisites
- Rust (stable)
- Docker (optional for local Nitro dev node)
- `cargo install stylus-trace-studio`

## 10-minute path
1. Create starter scaffold:
   ```bash
   cp -R templates/starter-repo stylus-trace-starter
   cd stylus-trace-starter
   ```
2. Generate profile artifacts from bundled sample data:
   ```bash
   ./scripts/run-local-profile.sh
   ```
3. Inspect output:
   - `artifacts/diff/diff.svg`
   - `artifacts/diff/diff_report.json`

## Live transaction profiling (optional)
```bash
stylus-trace capture \
  --rpc http://localhost:8547 \
  --tx 0xYOUR_TX_HASH \
  --output artifacts/capture/current_profile.json \
  --flamegraph artifacts/capture/capture.svg \
  --summary
```

Then compare to baseline:
```bash
stylus-trace diff \
  artifacts/capture/baseline.json \
  artifacts/capture/current_profile.json \
  --output artifacts/diff/diff_report.json \
  --flamegraph artifacts/diff/diff.svg \
  --summary
```
