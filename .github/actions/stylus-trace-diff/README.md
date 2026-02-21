# Stylus Trace Diff Action

This GitHub Action profiles Arbitrum Stylus transactions and automatically detects performance regressions.

## 🚀 Features

- **Automated Profiling**: Runs `stylus-trace capture` in CI.
- **Regression Gating**: Fails the build if gas usage exceeds a threshold.
- **Visual Feedback**: Generates flamegraphs and text summaries.
- **Baseline Comparison**: Seamlessly compares PRs against a cached baseline from your main branch.

## 📥 Inputs

| Input | Description | Required | Default |
| :--- | :--- | :--- | :--- |
| `tx_hash` | **(Required)** The transaction hash to profile. | Yes | - |
| `rpc_url` | The RPC endpoint of your Nitro node. | No | `http://localhost:8547` |
| `baseline` | Path to a baseline profile JSON for comparison. | No | - |
| `threshold_percent` | Percentage gas increase allowed before failing. | No | - |
| `ink` | Whether to display results in Stylus Ink units (10,000x). | No | `false` |

## 🛠 Usage Example

To use this action, you first need to generate a transaction hash by deploying and "exercising" your contract.

```yaml
jobs:
  profile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # 1. Start your Nitro Node (e.g., using OffchainLabs/nitro-devnode)
      - name: Start Nitro Node
        run: |
          docker run --rm -d --name nitro -p 8547:8547 offchainlabs/nitro-node:latest --dev --http.addr 0.0.0.0
          until curl -s http://localhost:8547; do sleep 5; done

      # 2. Deploy & Execute (The "Exerciser")
      - name: Deploy and Mint
        id: tx
        run: |
          # Use cast or your own scripts to get a TX hash
          HASH=$(cast send --rpc-url http://localhost:8547 --private-key ${{ secrets.SK }} $CONTRACT_ADDR "mint()" --json | jq -r '.transactionHash')
          echo "hash=$HASH" >> $GITHUB_OUTPUT

      # 3. Restore Baseline Profile
      - name: Restore Baseline
        uses: actions/cache@v4
        with:
          path: artifacts/capture/profile.json
          key: stylus-baseline-${{ github.base_ref || github.ref_name }}

      # 4. Run Stylus Trace
      - name: Profile & Diff
        uses: ./.github/actions/stylus-trace-diff
        with:
          tx_hash: ${{ steps.tx.outputs.hash }}
          baseline: artifacts/capture/profile.json
          threshold_percent: 5.0  # Fail if gas usage increases > 5%

      # 5. Save the new baseline (only on main branch)
      - name: Update Baseline
        if: github.ref == 'refs/heads/main'
        uses: actions/cache/save@v4
        with:
          path: artifacts/capture/profile.json
          key: stylus-baseline-main
```

## 📊 Regression Gating

The action uses `thresholds.toml` in your project root for fine-grained control. If the diff exceeds these values, the action returns exit code `1`, which stops your CI pipeline.

Example `thresholds.toml`:

```toml
[gas]
max_increase_percent = 5.0

[hostio]
max_total_calls_increase_percent = 10.0
```
