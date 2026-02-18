//! Core diff engine implementation.
//! Generates complete diff reports by comparing two profiles.

use crate::parser::schema::Profile;
use chrono::Utc;

use super::normalizer::{
    are_profiles_identical, calculate_gas_delta, calculate_hostio_delta, check_compatibility,
    compare_hot_paths,
};
use super::schema::{Deltas, DiffReport, DiffSummary, ProfileMetadata};
use super::DiffError;

/// Generate a complete diff report comparing two profiles
///
/// # Arguments
/// * `baseline` - The baseline profile to compare against
/// * `target` - The target profile to compare
///
/// # Returns
/// Complete DiffReport with all deltas calculated
///
/// # Errors
/// * `DiffError::IncompatibleVersions` - If schema versions don't match
///
/// # Example
/// ```ignore
/// use stylus_trace_studio::diff::generate_diff;
/// use stylus_trace_studio::output::json::read_profile;
///
/// let baseline = read_profile("baseline.json")?;
/// let target = read_profile("target.json")?;
/// let diff = generate_diff(&baseline, &target)?;
/// ```
pub fn generate_diff(baseline: &Profile, target: &Profile) -> Result<DiffReport, DiffError> {
    // Step 1: Check compatibility
    check_compatibility(baseline, target)?;

    // Step 2: Extract metadata
    let baseline_meta = ProfileMetadata {
        transaction_hash: baseline.transaction_hash.clone(),
        total_gas: baseline.total_gas,
        generated_at: baseline.generated_at.clone(),
    };

    let target_meta = ProfileMetadata {
        transaction_hash: target.transaction_hash.clone(),
        total_gas: target.total_gas,
        generated_at: target.generated_at.clone(),
    };

    // Step 3: Calculate all deltas
    let gas_delta = calculate_gas_delta(baseline.total_gas, target.total_gas);

    let hostio_delta = calculate_hostio_delta(&baseline.hostio_summary, &target.hostio_summary);

    let hot_paths_delta = compare_hot_paths(&baseline.hot_paths, &target.hot_paths);

    let deltas = Deltas {
        gas: gas_delta,
        hostio: hostio_delta,
        hot_paths: hot_paths_delta,
    };

    // Step 4: Create summary (no thresholds yet)
    let mut summary = DiffSummary {
        has_regressions: false,
        violation_count: 0,
        status: "PASSED".to_string(),
        warning: None,
    };

    // Check if profiles are identical
    if are_profiles_identical(baseline, target) {
        summary.warning = Some("Baseline and target profiles are identical".to_string());
    }

    // Step 5: Build the report
    Ok(DiffReport {
        diff_version: "1.0.0".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        baseline: baseline_meta,
        target: target_meta,
        deltas,
        threshold_violations: Vec::new(), // Will be populated by check_thresholds
        summary,
    })
}
