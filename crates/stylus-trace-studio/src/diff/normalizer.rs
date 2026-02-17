//! Profile normalization and delta calculation.
//!
//! Handles the math for computing differences between profiles,
//! including edge cases like division by zero.

use crate::parser::schema::{HotPath, HostIoSummary, Profile};
use std::collections::HashMap;

use super::schema::{
    GasDelta, HostIOTypeChange, HostIoDelta, HotPathComparison, HotPathsDelta,
};

/// Calculate gas delta between two profiles
///
/// # Arguments
/// * `baseline` - Baseline gas usage
/// * `target` - Target gas usage
///
/// # Returns
/// GasDelta with absolute and percentage changes
pub fn calculate_gas_delta(baseline: u64, target: u64) -> GasDelta {
    let absolute_change = (target as i64) - (baseline as i64);
    let percent_change = safe_percentage(absolute_change, baseline);

    GasDelta {
        baseline,
        target,
        absolute_change,
        percent_change,
    }
}

/// Calculate HostIO delta between two profiles
///
/// # Arguments
/// * `baseline_summary` - Baseline HostIO summary
/// * `target_summary` - Target HostIO summary
///
/// # Returns
/// HostIoDelta with all HostIO-related changes
pub fn calculate_hostio_delta(
    baseline_summary: &HostIoSummary,
    target_summary: &HostIoSummary,
) -> HostIoDelta {
    // Total calls delta
    let baseline_total_calls = baseline_summary.total_calls;
    let target_total_calls = target_summary.total_calls;
    let total_calls_change = (target_total_calls as i64) - (baseline_total_calls as i64);
    let total_calls_percent_change = safe_percentage(total_calls_change, baseline_total_calls);

    // HostIO gas delta
    let baseline_total_gas = baseline_summary.total_hostio_gas;
    let target_total_gas = target_summary.total_hostio_gas;
    let gas_change = (target_total_gas as i64) - (baseline_total_gas as i64);
    let gas_percent_change = safe_percentage(gas_change, baseline_total_gas);

    // By-type changes
    let by_type_changes = calculate_hostio_type_changes(
        &baseline_summary.by_type,
        &target_summary.by_type,
    );

    HostIoDelta {
        baseline_total_calls,
        target_total_calls,
        total_calls_change,
        total_calls_percent_change,
        by_type_changes,
        baseline_total_gas,
        target_total_gas,
        gas_change,
        gas_percent_change,
    }
}

/// Calculate changes for each HostIO type
///
/// Handles missing types by treating them as 0
pub fn calculate_hostio_type_changes(
    baseline_types: &HashMap<String, u64>,
    target_types: &HashMap<String, u64>,
) -> HashMap<String, HostIOTypeChange> {
    let mut changes = HashMap::new();

    // Collect all unique HostIO types from both profiles
    let mut all_types: std::collections::HashSet<String> = std::collections::HashSet::new();
    all_types.extend(baseline_types.keys().cloned());
    all_types.extend(target_types.keys().cloned());

    // Calculate delta for each type
    for hostio_type in all_types {
        let baseline = *baseline_types.get(&hostio_type).unwrap_or(&0);
        let target = *target_types.get(&hostio_type).unwrap_or(&0);
        let delta = (target as i64) - (baseline as i64);

        // Only include if there's a change or if it exists in either profile
        if delta != 0 || baseline > 0 || target > 0 {
            changes.insert(
                hostio_type,
                HostIOTypeChange {
                    baseline,
                    target,
                    delta,
                },
            );
        }
    }

    changes
}

/// Compare hot paths between two profiles
///
/// # Arguments
/// * `baseline_paths` - Hot paths from baseline
/// * `target_paths` - Hot paths from target
///
/// # Returns
/// HotPathsDelta showing common, disappeared, and new paths
pub fn compare_hot_paths(
    baseline_paths: &[HotPath],
    target_paths: &[HotPath],
) -> HotPathsDelta {
    // Create maps for easier lookup
    let baseline_map: HashMap<&str, &HotPath> = baseline_paths
        .iter()
        .map(|hp| (hp.stack.as_str(), hp))
        .collect();

    let target_map: HashMap<&str, &HotPath> = target_paths
        .iter()
        .map(|hp| (hp.stack.as_str(), hp))
        .collect();

    // Find common paths
    let mut common_paths = Vec::new();
    for (stack, baseline_path) in &baseline_map {
        if let Some(target_path) = target_map.get(stack) {
            let baseline_gas = baseline_path.gas;
            let target_gas = target_path.gas;
            let gas_change = (target_gas as i64) - (baseline_gas as i64);
            let percent_change = safe_percentage(gas_change, baseline_gas);

            common_paths.push(HotPathComparison {
                stack: stack.to_string(),
                baseline_gas,
                target_gas,
                gas_change,
                percent_change,
            });
        }
    }

    // Find paths only in baseline (disappeared)
    let baseline_only: Vec<HotPath> = baseline_paths
        .iter()
        .filter(|hp| !target_map.contains_key(hp.stack.as_str()))
        .cloned()
        .collect();

    // Find paths only in target (new)
    let target_only: Vec<HotPath> = target_paths
        .iter()
        .filter(|hp| !baseline_map.contains_key(hp.stack.as_str()))
        .cloned()
        .collect();

    HotPathsDelta {
        common_paths,
        baseline_only,
        target_only,
    }
}

/// Calculate percentage change safely (handles division by zero)
///
/// # Arguments
/// * `change` - Absolute change (can be negative)
/// * `baseline` - Baseline value
///
/// # Returns
/// Percentage change, or 0.0 if baseline is zero
pub fn safe_percentage(change: i64, baseline: u64) -> f64 {
    if baseline == 0 {
        // If baseline is 0, we can't calculate percentage
        // Return 0.0 as a safe default
        0.0
    } else {
        (change as f64 / baseline as f64) * 100.0
    }
}

/// Check if two profiles are compatible for comparison
///
/// # Arguments
/// * `baseline` - Baseline profile
/// * `target` - Target profile
///
/// # Returns
/// Ok if compatible, Err with reason if not
pub fn check_compatibility(
    baseline: &Profile,
    target: &Profile,
) -> Result<(), super::DiffError> {
    // Check version compatibility
    if baseline.version != target.version {
        return Err(super::DiffError::IncompatibleVersions(
            baseline.version.clone(),
            target.version.clone(),
        ));
    }

    Ok(())
}

/// Check if profiles are identical
///
/// # Arguments
/// * `baseline` - Baseline profile
/// * `target` - Target profile
///
/// # Returns
/// true if the profiles have identical transaction hashes
pub fn are_profiles_identical(baseline: &Profile, target: &Profile) -> bool {
    baseline.transaction_hash == target.transaction_hash
        && baseline.total_gas == target.total_gas
}
