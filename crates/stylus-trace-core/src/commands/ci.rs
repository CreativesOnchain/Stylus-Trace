//! CI command implementation.
//! Handles automated CI/CD configuration for external projects.

use crate::commands::models::CiInitArgs;
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::Path;

/// Execute the CI init command
pub fn execute_ci_init(args: CiInitArgs) -> Result<()> {
    // 1. Ensure .github/workflows exists
    let workflow_dir = Path::new(".github/workflows");
    if !workflow_dir.exists() {
        fs::create_dir_all(workflow_dir).context("Failed to create .github/workflows directory")?;
    }

    let workflow_path = workflow_dir.join("performance.yml");
    if workflow_path.exists() && !args.force {
        println!(
            "{}",
            "⚠️  .github/workflows/performance.yml already exists. Use --force to overwrite."
                .yellow()
        );
        return Ok(());
    }

    // 2. Generate YAML

    let tx_hash = args
        .transaction_hash
        .as_deref()
        .unwrap_or("YOUR_TRANSACTION_HASH");

    let workflow_yaml = format!(
        r#"name: Stylus Performance Check

on:
  pull_request:
    branches: [ main ]
  push:
    branches: [ main ]

jobs:
  performance-regression:
    name: Gas Regression Check
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Prepare Profiles
        run: |
          mkdir -p artifacts/capture
          # If paths exist, stage them for the check
          [ -f "artifacts/capture/baseline.json" ] || echo "{{}}" > artifacts/capture/baseline.json
          [ -f "artifacts/capture/current_profile.json" ] || cp artifacts/capture/baseline.json artifacts/capture/current_profile.json

      - name: Run Stylus Performance Check
        uses: CreativesOnchain/Stylus-Trace@main
        with:
          tx_hash: "{}"
          threshold: "{}"
          skip_capture: "true"
"#,
        tx_hash, args.threshold
    );

    // 3. Write file
    fs::write(&workflow_path, workflow_yaml).context("Failed to write performance.yml")?;

    println!(
        "{} {}",
        "✅ Created".green(),
        workflow_path.display().to_string().cyan()
    );
    println!(
        "{}",
        "💡 Important: Ensure you have captured a baseline profile and committed it to 'artifacts/capture/baseline.json'".dimmed()
    );

    Ok(())
}
