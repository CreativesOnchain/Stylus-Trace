//! Comprehensive consolidated tests for the diff module.
//!
//! Includes all integration and unit tests for the diffing functionality.

use stylus_trace_studio::diff::*;
use stylus_trace_studio::parser::schema::{HotPath, HostIoSummary, Profile};
use std::collections::HashMap;

// ============================================================================
// SHARED TEST HELPERS
// ============================================================================

fn create_full_test_profile(
    tx_hash: &str,
    version: &str,
    total_gas: u64,
    hostio_total_calls: u64,
    hostio_by_type: HashMap<String, u64>,
    hostio_total_gas: u64,
    hot_paths: Vec<HotPath>,
) -> Profile {
    Profile {
        version: version.to_string(),
        transaction_hash: tx_hash.to_string(),
        total_gas,
        hostio_summary: HostIoSummary {
            total_calls: hostio_total_calls,
            by_type: hostio_by_type,
            total_hostio_gas: hostio_total_gas,
        },
        hot_paths,
        generated_at: "2025-02-14T10:00:00Z".to_string(),
    }
}

// ============================================================================
// COMPONENT TESTS: ENGINE
// ============================================================================

mod engine_tests {
    use super::*;

    fn create_p(tx: &str, gas: u64) -> Profile {
        create_full_test_profile(tx, "1.0.0", gas, 0, HashMap::new(), 0, vec![])
    }

    #[test]
    fn test_generate_diff_basic() {
        let b = create_p("0x1", 100);
        let t = create_p("0x2", 150);
        let diff = generate_diff(&b, &t).unwrap();
        assert_eq!(diff.deltas.gas.percent_change, 50.0);
    }

    #[test]
    fn test_generate_diff_identical() {
        let b = create_p("0x1", 100);
        let t = b.clone();
        let diff = generate_diff(&b, &t).unwrap();
        assert!(diff.summary.warning.is_some());
    }

    #[test]
    fn test_generate_diff_incompatible() {
        let mut b = create_p("0x1", 100);
        let mut t = create_p("0x2", 150);
        b.version = "1.0.0".to_string();
        t.version = "2.0.0".to_string();
        assert!(generate_diff(&b, &t).is_err());
    }
}

// ============================================================================
// COMPONENT TESTS: NORMALIZER
// ============================================================================

mod normalizer_tests {
    use super::*;

    #[test]
    fn test_safe_percentage_logic() {
        assert_eq!(safe_percentage(50, 100), 50.0);
        assert_eq!(safe_percentage(10, 0), 0.0);
    }

    #[test]
    fn test_calculate_gas_delta_logic() {
        let d = calculate_gas_delta(150, 100);
        assert_eq!(d.absolute_change, -50);
        assert_eq!(d.percent_change, -33.33333333333333);
    }

    #[test]
    fn test_hostio_type_changes_logic() {
        let mut b = HashMap::new();
        b.insert("load".to_string(), 10);
        let mut t = HashMap::new();
        t.insert("store".to_string(), 5);
        let changes = calculate_hostio_type_changes(&b, &t);
        assert_eq!(changes.get("load").unwrap().delta, -10);
        assert_eq!(changes.get("store").unwrap().delta, 5);
    }
}

// ============================================================================
// COMPONENT TESTS: OUTPUT
// ============================================================================

mod output_tests {
    use super::*;

    #[test]
    fn test_render_terminal_diff_basic() {
        let report = DiffReport {
            diff_version: "1.0.0".to_string(),
            generated_at: "now".to_string(),
            baseline: ProfileMetadata { transaction_hash: "0x1".to_string(), total_gas: 100, generated_at: "now".to_string() },
            target: ProfileMetadata { transaction_hash: "0x2".to_string(), total_gas: 120, generated_at: "now".to_string() },
            deltas: Deltas {
                gas: GasDelta { baseline: 100, target: 120, absolute_change: 20, percent_change: 20.0 },
                hostio: HostIoDelta::default(),
                hot_paths: HotPathsDelta::default(),
            },
            threshold_violations: vec![],
            summary: DiffSummary { status: "FAILED".to_string(), violation_count: 1, has_regressions: true, warning: None },
        };
        let out = render_terminal_diff(&report);
        assert!(out.contains("Total Gas: 100 -> 120 (+20.00%)"));
    }
}

// ============================================================================
// COMPONENT TESTS: THRESHOLD
// ============================================================================

mod threshold_tests {
    use super::*;

    #[test]
    fn test_gas_threshold_exceeded_logic() {
        let delta = GasDelta { baseline: 100, target: 150, absolute_change: 50, percent_change: 50.0 };
        let thresholds = GasThresholds { max_increase_percent: Some(10.0), ..Default::default() };
        let mut v = vec![];
        check_gas_thresholds(&delta, &thresholds, &mut v);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn test_create_summary_logic() {
        let v = vec![ThresholdViolation { metric: "m".to_string(), severity: "error".to_string(), ..Default::default() }];
        assert_eq!(create_summary(&v).status, "FAILED");
        let v2 = vec![ThresholdViolation { metric: "m".to_string(), severity: "warning".to_string(), ..Default::default() }];
        assert_eq!(create_summary(&v2).status, "WARNING");
    }
}

// ============================================================================
// COMPLEX INTEGRATION SCENARIOS
// ============================================================================

#[test]
fn test_complex_regression_scenario() {
    let mut b_types = HashMap::new();
    b_types.insert("storage_load".to_string(), 10);
    let baseline = create_full_test_profile("0x1", "1.0.0", 100000, 10, b_types, 1000, vec![]);

    let mut t_types = HashMap::new();
    t_types.insert("storage_load".to_string(), 20);
    let target = create_full_test_profile("0x2", "1.0.0", 200000, 20, t_types, 2000, vec![]);

    let mut diff = generate_diff(&baseline, &target).unwrap();
    
    let mut limits = HashMap::new();
    limits.insert("storage_load".to_string(), 5);
    let config = ThresholdConfig {
        gas: GasThresholds { max_increase_percent: Some(10.0), ..Default::default() },
        hostio: HostIOThresholds { limits: Some(limits), ..Default::default() },
        ..Default::default()
    };

    let v = check_thresholds(&mut diff, &config);
    assert!(v.iter().any(|violation| violation.metric == "gas.max_increase_percent"));
    assert!(v.iter().any(|violation| violation.metric.contains("storage_load")));
}

#[test]
fn test_hot_paths_comparison_logic() {
    let b_paths = vec![HotPath { stack: "A;B".to_string(), gas: 100, percentage: 50.0, source_hint: None }];
    let t_paths = vec![HotPath { stack: "A;B".to_string(), gas: 150, percentage: 75.0, source_hint: None }];
    
    let b = create_full_test_profile("0x1", "1.0.0", 200, 0, HashMap::new(), 0, b_paths);
    let t = create_full_test_profile("0x2", "1.0.0", 200, 0, HashMap::new(), 0, t_paths);
    
    let diff = generate_diff(&b, &t).unwrap();
    assert_eq!(diff.deltas.hot_paths.common_paths.len(), 1);
    assert_eq!(diff.deltas.hot_paths.common_paths[0].percent_change, 50.0);
}
