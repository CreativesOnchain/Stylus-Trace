//! Diff command implementation.
//! Orchestrates the comparison of two profiles and reports deltas/violations.

use super::models::DiffArgs;
use crate::diff::{
    check_thresholds, generate_diff, load_thresholds, render_terminal_diff, GasThresholds,
    HostIOThresholds, ThresholdConfig,
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
        // Auto-load thresholds.toml from CWD if it exists
        let auto_path = std::path::Path::new("thresholds.toml");
        if auto_path.exists() {
            load_thresholds(auto_path)
                .context("Failed to auto-load thresholds.toml from project root")?
        } else {
            ThresholdConfig::default()
        }
    };

    // Override with simple percent if provided (Simple Mode)
    if let Some(percent) = args.threshold_percent {
        // Enforce strict overrides: clear granular limits and absolute values
        thresholds.gas.max_increase_percent = Some(percent);
        thresholds.gas.max_increase_absolute = None;

        thresholds.hostio.max_total_calls_increase_percent = Some(percent);
        thresholds.hostio.limits = None;

        thresholds.hot_paths = Some(crate::diff::HotPathThresholds {
            warn_individual_increase_percent: Some(percent),
        });
    }

    // Handle granular overrides (highest precedence)
    // If a granular flag is used without a global percent, we enter "Focus Mode"
    // and disable other categories that weren't explicitly requested.
    let has_global = args.threshold_percent.is_some();
    let has_gas = args.gas_threshold.is_some();
    let has_hostio = args.hostio_threshold.is_some();

    if has_gas {
        thresholds.gas.max_increase_percent = args.gas_threshold;
        thresholds.gas.max_increase_absolute = None;

        // If focusing specifically on gas, disable hostio/hotpaths unless they were also specified
        if !has_global && !has_hostio {
            thresholds.hostio = HostIOThresholds::default();
            thresholds.hot_paths = None;
        }
    }

    if has_hostio {
        thresholds.hostio.max_total_calls_increase_percent = args.hostio_threshold;
        thresholds.hostio.limits = None;

        // If focusing specifically on hostio, disable gas/hotpaths unless they were also specified
        if !has_global && !has_gas {
            thresholds.gas = GasThresholds::default();
            thresholds.hot_paths = None;
        }
    }

    // Step 4: Check violations
    check_thresholds(&mut report, &thresholds);

    // Step 5: Write output if requested
    if let Some(path) = &args.output {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .context("Failed to create parent directories for diff report")?;
            }
        }

        let json = serde_json::to_string_pretty(&report)?;
        fs::write(path, json).context("Failed to write diff report JSON")?;
        println!(
            "📊 Diff report written to {}",
            path.display().to_string().cyan()
        );
    }

    if let Some(path) = &args.output_svg {
        let baseline_stacks = baseline.all_stacks.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Baseline profile missing full execution stacks. Please re-capture.")
        })?;
        let target_stacks = target.all_stacks.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Target profile missing full execution stacks. Please re-capture.")
        })?;

        let svg = crate::flamegraph::generate_diff_flamegraph(baseline_stacks, target_stacks, None)
            .context("Failed to generate diff flamegraph")?;

        crate::output::svg::write_svg(&svg, path).context("Failed to write diff flamegraph SVG")?;
        println!(
            "🔥 Visual diff written to {}",
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
