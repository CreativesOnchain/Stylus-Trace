# Lesson Plan: Profiling Stylus Contracts with Stylus Trace

## Audience
- Solidity or Rust smart contract developers new to Stylus performance analysis

## Duration
- 60 minutes

## Learning outcomes
- Understand `capture`, `diff`, and artifact outputs
- Identify hot paths from flamegraphs
- Apply threshold gating in CI

## Agenda
1. 10 min: Why profiling matters in Stylus
2. 15 min: Run first profile and read summary
3. 20 min: Compare baseline vs target with `stylus-trace diff`
4. 10 min: Add CI gating thresholds
5. 5 min: Review and Q&A

## Hands-on exercise
1. Run:
   ```bash
   stylus-trace diff baseline.json current_profile.json --flamegraph diff.svg --output diff_report.json
   ```
2. Open `diff.svg` and identify the largest regression.
3. Tune `thresholds.toml` to fail only meaningful regressions.

## Assessment rubric
- Can explain top 3 cost drivers from a profile
- Can configure threshold gates in CI
- Can interpret a machine-readable diff report
