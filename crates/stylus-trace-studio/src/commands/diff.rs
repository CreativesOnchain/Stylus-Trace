//! Diff command implementation.
//! Orchestrates the comparison of two profiles and reports deltas/violations.

use super::models::DiffArgs;
use crate::diff::{
    check_thresholds, generate_diff, load_thresholds, render_terminal_diff, GasThresholds,
    ThresholdConfig,
};
use crate::output::json::read_profile;
use crate::parser::schema::Profile;
use anyhow::{Context, Result};
use colored::*;
use std::fs;

/// Execute the diff command
pub fn execute_diff(args: DiffArgs) -> Result<()> {
    // Step 1: Load profiles
    let baseline: Profile =
        read_profile(&args.baseline).context("Failed to read baseline profile")?;
    let target: Profile = read_profile(&args.target).context("Failed to read target profile")?;

    // Step 2: Generate diff
    let mut report = generate_diff(&baseline, &target).context("Failed to generate diff")?;

    // Step 3: Handle thresholds
    let mut thresholds = if let Some(path) = &args.threshold_file {
        load_thresholds(path).context("Failed to load threshold file")?
    } else {
        ThresholdConfig::default()
    };

    // Override with simple percent if provided
    if let Some(percent) = args.threshold_percent {
        thresholds.gas = GasThresholds {
            max_increase_percent: Some(percent),
            max_increase_absolute: None,
        };
    }

    // Step 4: Check violations only if thresholds are set
    if args.threshold_file.is_some() || args.threshold_percent.is_some() {
        check_thresholds(&mut report, &thresholds);
    }

    // Step 5: Write output if requested
    if let Some(path) = &args.output {
        let json = serde_json::to_string_pretty(&report)?;
        fs::write(path, json).context("Failed to write diff report JSON")?;
        println!(
            "📊 Diff report written to {}",
            path.display().to_string().cyan()
        );
    }

    // Step 6: Terminal Summary
    if args.summary {
        println!("{}", render_terminal_diff(&report));
    }

    // Step 7: Final Status Exit Code Handling (implicit)
    if report.summary.status == "FAILED" {
        return Err(anyhow::anyhow!("Regression detected against thresholds"));
    }

    Ok(())
}
