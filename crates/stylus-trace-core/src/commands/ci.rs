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
    let rpc_line = if let Some(rpc) = &args.rpc_url {
        format!("          rpc_url: \"{}\"\n", rpc)
    } else {
        String::new()
    };

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

      - name: Run Stylus Performance Check
        uses: CreativesOnchain/Stylus-Trace@main
        with:
          tx_hash: "{}"
{}          threshold: "{}"
"#,
        args.transaction_hash, rpc_line, args.threshold
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
