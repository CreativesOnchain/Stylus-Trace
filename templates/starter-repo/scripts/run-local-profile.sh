#!/usr/bin/env sh
set -eu

mkdir -p artifacts/capture artifacts/diff
cp profiles/baseline.json artifacts/capture/baseline.json
cp profiles/current_profile.json artifacts/capture/current_profile.json

stylus-trace diff \
  artifacts/capture/baseline.json \
  artifacts/capture/current_profile.json \
  --output artifacts/diff/diff_report.json \
  --flamegraph artifacts/diff/diff.svg \
  --summary

echo "Artifacts generated in artifacts/diff"
