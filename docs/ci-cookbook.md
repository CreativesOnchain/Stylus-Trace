# CI Cookbook

## Goal
Keep CI green by default, always upload profiling artifacts, and gate regressions when thresholds are exceeded.

## Starter workflow pattern
1. Stage baseline/current profile inputs.
2. Optionally run live capture with repo variables.
3. Run `stylus-trace diff`.
4. Upload `artifacts/` with `actions/upload-artifact`.

## Baseline gate examples
- Global gate:
  ```bash
  stylus-trace diff baseline.json current_profile.json --threshold-percent 3.0
  ```
- Gas-only gate:
  ```bash
  stylus-trace diff baseline.json current_profile.json --gas-threshold 1.5
  ```
- HostIO-only gate:
  ```bash
  stylus-trace diff baseline.json current_profile.json --hostio-threshold 5.0
  ```

## Cache guidance
- Cache Cargo index and target with `Swatinem/rust-cache@v2`.
- Keep baseline profiles committed to repo for deterministic first runs.

## Artifact set
- `artifacts/capture/current_profile.json`
- `artifacts/diff/diff_report.json`
- `artifacts/diff/diff.svg`
