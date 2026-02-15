//! Core diff engine implementation.
//!
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

    let hostio_delta =
        calculate_hostio_delta(&baseline.hostio_summary, &target.hostio_summary);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::schema::{HotPath, HostIoSummary};
    use std::collections::HashMap;

    fn create_test_profile(
        tx_hash: &str,
        total_gas: u64,
        hostio_calls: u64,
    ) -> Profile {
        let mut by_type = HashMap::new();
        by_type.insert("storage_load".to_string(), 10);

        Profile {
            version: "1.0.0".to_string(),
            transaction_hash: tx_hash.to_string(),
            total_gas,
            hostio_summary: HostIoSummary {
                total_calls: hostio_calls,
                by_type,
                total_hostio_gas: 45000,
            },
            hot_paths: vec![HotPath {
                stack: "main;func".to_string(),
                gas: 1000,
                percentage: 10.0,
                source_hint: None,
            }],
            generated_at: "2025-02-14T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_generate_diff_basic() {
        let baseline = create_test_profile("0xaaa", 100000, 25);
        let target = create_test_profile("0xbbb", 150000, 35);

        let diff = generate_diff(&baseline, &target).unwrap();

        assert_eq!(diff.baseline.total_gas, 100000);
        assert_eq!(diff.target.total_gas, 150000);
        assert_eq!(diff.deltas.gas.absolute_change, 50000);
        assert_eq!(diff.deltas.gas.percent_change, 50.0);
        assert_eq!(diff.summary.status, "PASSED");
    }

    #[test]
    fn test_generate_diff_identical_profiles() {
        let baseline = create_test_profile("0xaaa", 100000, 25);
        let target = baseline.clone();

        let diff = generate_diff(&baseline, &target).unwrap();

        assert!(diff.summary.warning.is_some());
        assert!(diff
            .summary
            .warning
            .unwrap()
            .contains("identical"));
    }

    #[test]
    fn test_generate_diff_incompatible_versions() {
        let mut baseline = create_test_profile("0xaaa", 100000, 25);
        let mut target = create_test_profile("0xbbb", 150000, 35);

        baseline.version = "1.0.0".to_string();
        target.version = "2.0.0".to_string();

        let result = generate_diff(&baseline, &target);
        assert!(result.is_err());

        if let Err(DiffError::IncompatibleVersions(v1, v2)) = result {
            assert_eq!(v1, "1.0.0");
            assert_eq!(v2, "2.0.0");
        } else {
            panic!("Expected IncompatibleVersions error");
        }
    }

    #[test]
    fn test_generate_diff_improvement() {
        let baseline = create_test_profile("0xaaa", 150000, 35);
        let target = create_test_profile("0xbbb", 100000, 25);

        let diff = generate_diff(&baseline, &target).unwrap();

        assert_eq!(diff.deltas.gas.absolute_change, -50000);
        assert!(diff.deltas.gas.percent_change < 0.0);
    }
}