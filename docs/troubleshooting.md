# Troubleshooting

## `debug_traceTransaction` returns method not found
- RPC endpoint does not expose tracing methods.
- Use a Nitro dev node or a provider tier that enables debug APIs.

## `capture` fails for valid hash
- Ensure the hash exists on the same chain as your RPC endpoint.
- Confirm transaction has finalized.

## Diff command fails on profile schema
- Verify both files are Stylus Trace profile JSON outputs.
- Check schema version compatibility.

## No flamegraph output in CI
- Confirm `--flamegraph` path is passed to `stylus-trace diff` or `capture`.
- Confirm artifacts are uploaded from the same path.

## CI green locally but red on GitHub
- Check missing repo vars/secrets for optional live capture.
- Keep fallback fixture profiles in repo for deterministic output.
