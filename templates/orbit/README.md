# Orbit Template: Custom Chain Profiling

This template demonstrates profiling on an Orbit chain with a custom chain ID and RPC.

## Included
- `.github/workflows/orbit-performance.yml`
- `orbit.env.example`

## Required repository variables
- `ORBIT_RPC_URL`
- `ORBIT_TX_HASH`

## Required repository secrets
- `ORBIT_CHAIN_ID` (example: `412346`)

The workflow captures and diffs a profile, then uploads artifacts for PR review.
