//! Integration tests for the diff module.
//!
//! Tests the complete diff generation and threshold checking workflow.

use super::*;
use crate::parser::schema::{HotPath, HostIoSummary, Profile};
use std::collections::HashMap;

/// Helper function to create a test profile
fn create_test_profile(
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

/// Create baseline profile matching examples/profile-baseline.json
fn create_baseline_profile() -> Profile {
    let mut by_type = HashMap::new();
    by_type.insert("storage_load".to_string(), 10);
    by_type.insert("storage_store".to_string(), 5);
    by_type.insert("call".to_string(), 3);
    by_type.insert("emit_log".to_string(), 2);
    by_type.insert("native_keccak256".to_string(), 5);

    let hot_paths = vec![
        HotPath {
            stack: "contract_main;process_transaction;validate_signature".to_string(),
            gas: 35000,
            percentage: 23.33,
            source_hint: None,
        },
        HotPath {
            stack: "contract_main;process_transaction;check_balance;storage_load".to_string(),
            gas: 28000,
            percentage: 18.67,
            source_hint: None,
        },
        HotPath {
            stack: "contract_main;process_transaction;emit_event;emit_log".to_string(),
            gas: 15000,
            percentage: 10.0,
            source_hint: None,
        },
    ];

    create_test_profile(
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "1.0.0",
        150000,
        25,
        by_type,
        45000,
        hot_paths,
    )
}

/// Create regression profile matching examples/profile-regression.json
fn create_regression_profile() -> Profile {
    let mut by_type = HashMap::new();
    by_type.insert("storage_load".to_string(), 22);
    by_type.insert("storage_store".to_string(), 8);
    by_type.insert("call".to_string(), 3);
    by_type.insert("emit_log".to_string(), 3);
    by_type.insert("native_keccak256".to_string(), 6);

    let hot_paths = vec![
        HotPath {
            stack: "contract_main;process_transaction;validate_signature".to_string(),
            gas: 38000,
            percentage: 18.10,
            source_hint: None,
        },
        HotPath {
            stack: "contract_main;process_transaction;check_balance;storage_load".to_string(),
            gas: 45000,
            percentage: 21.43,
            source_hint: None,
        },
        HotPath {
            stack: "contract_main;process_transaction;redundant_check;storage_load".to_string(),
            gas: 20000,
            percentage: 9.52,
            source_hint: None,
        },
    ];

    create_test_profile(
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd",
        "1.0.0",
        210000,
        42,
        by_type,
        72000,
        hot_paths,
    )
}

/// Create improvement profile matching examples/profile-improvement.json
fn create_improvement_profile() -> Profile {
    let mut by_type = HashMap::new();
    by_type.insert("storage_load".to_string(), 8);
    by_type.insert("storage_store".to_string(), 4);
    by_type.insert("call".to_string(), 2);
    by_type.insert("emit_log".to_string(), 1);
    by_type.insert("native_keccak256".to_string(), 3);

    let hot_paths = vec![
        HotPath {
            stack: "contract_main;process_transaction;validate_signature_optimized".to_string(),
            gas: 28000,
            percentage: 22.40,
            source_hint: None,
        },
        HotPath {
            stack: "contract_main;process_transaction;check_balance;storage_load".to_string(),
            gas: 22000,
            percentage: 17.60,
            source_hint: None,
        },
    ];

    create_test_profile(
        "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba",
        "1.0.0",
        125000,
        18,
        by_type,
        35000,
        hot_paths,
    )
}

/// Create example threshold configuration
fn create_test_thresholds() -> ThresholdConfig {
    let mut hostio_limits = HashMap::new();
    hostio_limits.insert("storage_load_max_increase".to_string(), 5);
    hostio_limits.insert("storage_store_max_increase".to_string(), 3);

    ThresholdConfig {
        gas: GasThresholds {
            max_increase_percent: Some(10.0),
            max_increase_absolute: Some(50000),
        },
        hostio: HostIOThresholds {
            max_total_calls_increase_percent: Some(20.0),
            limits: Some(hostio_limits),
        },
        hot_paths: Some(HotPathThresholds {
            warn_individual_increase_percent: Some(50.0),
        }),
    }
}

// ============================================================================
// TEST CASE 1: Regression Detection (MUST FAIL)
// ============================================================================

#[test]
fn test_regression_detection() {
    let baseline = create_baseline_profile();
    let target = create_regression_profile();

    let mut diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    // Verify gas delta
    assert_eq!(diff.deltas.gas.baseline, 150000);
    assert_eq!(diff.deltas.gas.target, 210000);
    assert_eq!(diff.deltas.gas.absolute_change, 60000);
    assert_eq!(diff.deltas.gas.percent_change, 40.0);

    // Verify HostIO delta
    assert_eq!(diff.deltas.hostio.baseline_total_calls, 25);
    assert_eq!(diff.deltas.hostio.target_total_calls, 42);
    assert_eq!(diff.deltas.hostio.total_calls_change, 17);
    assert_eq!(diff.deltas.hostio.total_calls_percent_change, 68.0);

    // Verify storage_load increased
    let storage_load_change = diff
        .deltas
        .hostio
        .by_type_changes
        .get("storage_load")
        .expect("storage_load should be present");
    assert_eq!(storage_load_change.baseline, 10);
    assert_eq!(storage_load_change.target, 22);
    assert_eq!(storage_load_change.delta, 12);

    // Apply thresholds
    let thresholds = create_test_thresholds();
    let violations = check_thresholds(&mut diff, &thresholds);

    // Should have violations
    assert!(!violations.is_empty(), "Should have threshold violations");
    assert_eq!(diff.summary.status, "FAILED");
    assert!(diff.summary.has_regressions);

    // Check specific violations
    let gas_violation = violations
        .iter()
        .find(|v| v.metric == "gas.max_increase_percent");
    assert!(gas_violation.is_some(), "Should violate gas threshold");
    assert_eq!(gas_violation.unwrap().threshold, 10.0);
    assert_eq!(gas_violation.unwrap().actual, 40.0);

    let hostio_violation = violations
        .iter()
        .find(|v| v.metric == "hostio.max_total_calls_increase_percent");
    assert!(
        hostio_violation.is_some(),
        "Should violate HostIO threshold"
    );
}

// ============================================================================
// TEST CASE 2: Improvement Detection (MUST PASS)
// ============================================================================

#[test]
fn test_improvement_detection() {
    let baseline = create_baseline_profile();
    let target = create_improvement_profile();

    let mut diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    // Verify gas improvement
    assert_eq!(diff.deltas.gas.baseline, 150000);
    assert_eq!(diff.deltas.gas.target, 125000);
    assert_eq!(diff.deltas.gas.absolute_change, -25000);
    assert!(diff.deltas.gas.percent_change < 0.0);

    // Verify HostIO improvement
    assert_eq!(diff.deltas.hostio.total_calls_change, -7);
    assert!(diff.deltas.hostio.total_calls_percent_change < 0.0);

    // Apply thresholds
    let thresholds = create_test_thresholds();
    let violations = check_thresholds(&mut diff, &thresholds);

    // Should have no violations
    assert_eq!(violations.len(), 0);
    assert_eq!(diff.summary.status, "PASSED");
    assert!(!diff.summary.has_regressions);
}

// ============================================================================
// TEST CASE 3: No Change (MUST PASS with warning)
// ============================================================================

#[test]
fn test_no_change() {
    let baseline = create_baseline_profile();
    let target = baseline.clone();

    let mut diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    // All deltas should be zero
    assert_eq!(diff.deltas.gas.absolute_change, 0);
    assert_eq!(diff.deltas.gas.percent_change, 0.0);
    assert_eq!(diff.deltas.hostio.total_calls_change, 0);

    // Should have warning about identical profiles
    assert!(diff.summary.warning.is_some());
    assert!(diff
        .summary
        .warning
        .unwrap()
        .contains("identical"));

    // Apply thresholds
    let thresholds = create_test_thresholds();
    let violations = check_thresholds(&mut diff, &thresholds);

    assert_eq!(violations.len(), 0);
    assert_eq!(diff.summary.status, "PASSED");
}

// ============================================================================
// TEST CASE 4: Version Incompatibility (MUST ERROR)
// ============================================================================

#[test]
fn test_version_incompatibility() {
    let mut baseline = create_baseline_profile();
    let mut target = create_regression_profile();

    baseline.version = "1.0.0".to_string();
    target.version = "1.1.0".to_string();

    let result = generate_diff(&baseline, &target);

    assert!(result.is_err());
    match result {
        Err(DiffError::IncompatibleVersions(v1, v2)) => {
            assert_eq!(v1, "1.0.0");
            assert_eq!(v2, "1.1.0");
        }
        _ => panic!("Expected IncompatibleVersions error"),
    }
}

// ============================================================================
// TEST CASE 5: Missing HostIO Categories (MUST HANDLE)
// ============================================================================

#[test]
fn test_missing_hostio_categories() {
    let mut baseline_types = HashMap::new();
    baseline_types.insert("storage_load".to_string(), 10);
    baseline_types.insert("storage_store".to_string(), 5);

    let mut target_types = HashMap::new();
    target_types.insert("storage_load".to_string(), 8);
    target_types.insert("call".to_string(), 5);

    let baseline = create_test_profile(
        "0xaaa",
        "1.0.0",
        100000,
        15,
        baseline_types,
        30000,
        vec![],
    );

    let target = create_test_profile(
        "0xbbb",
        "1.0.0",
        105000,
        13,
        target_types,
        28000,
        vec![],
    );

    let diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    // Verify storage_load decreased
    let storage_load = diff
        .deltas
        .hostio
        .by_type_changes
        .get("storage_load")
        .unwrap();
    assert_eq!(storage_load.baseline, 10);
    assert_eq!(storage_load.target, 8);
    assert_eq!(storage_load.delta, -2);

    // Verify storage_store disappeared (treated as 0)
    let storage_store = diff
        .deltas
        .hostio
        .by_type_changes
        .get("storage_store")
        .unwrap();
    assert_eq!(storage_store.baseline, 5);
    assert_eq!(storage_store.target, 0);
    assert_eq!(storage_store.delta, -5);

    // Verify call appeared (treated as 0 in baseline)
    let call = diff
        .deltas
        .hostio
        .by_type_changes
        .get("call")
        .unwrap();
    assert_eq!(call.baseline, 0);
    assert_eq!(call.target, 5);
    assert_eq!(call.delta, 5);
}

// ============================================================================
// TEST CASE 6: Zero Baseline (Division by Zero Protection)
// ============================================================================

#[test]
fn test_zero_baseline_no_panic() {
    let mut baseline_types = HashMap::new();
    baseline_types.insert("storage_load".to_string(), 0);

    let mut target_types = HashMap::new();
    target_types.insert("storage_load".to_string(), 10);

    let baseline = create_test_profile(
        "0xaaa",
        "1.0.0",
        100000,
        0,
        baseline_types,
        0,
        vec![],
    );

    let target = create_test_profile(
        "0xbbb",
        "1.0.0",
        150000,
        10,
        target_types,
        20000,
        vec![],
    );

    // Should not panic
    let diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    // Verify storage_load change
    let storage_load = diff
        .deltas
        .hostio
        .by_type_changes
        .get("storage_load")
        .unwrap();
    assert_eq!(storage_load.baseline, 0);
    assert_eq!(storage_load.target, 10);
    assert_eq!(storage_load.delta, 10);

    // Percentage with zero baseline should be 0.0 (safe default)
    assert_eq!(diff.deltas.hostio.total_calls_percent_change, 0.0);
}

// ============================================================================
// TEST CASE 7: Hot Paths Comparison
// ============================================================================

#[test]
fn test_hot_paths_comparison() {
    let baseline_paths = vec![
        HotPath {
            stack: "main;func_a".to_string(),
            gas: 100,
            percentage: 50.0,
            source_hint: None,
        },
        HotPath {
            stack: "main;func_b".to_string(),
            gas: 80,
            percentage: 40.0,
            source_hint: None,
        },
    ];

    let target_paths = vec![
        HotPath {
            stack: "main;func_a".to_string(),
            gas: 150,
            percentage: 60.0,
            source_hint: None,
        },
        HotPath {
            stack: "main;func_c".to_string(),
            gas: 70,
            percentage: 28.0,
            source_hint: None,
        },
    ];

    let baseline = create_test_profile(
        "0xaaa",
        "1.0.0",
        200,
        5,
        HashMap::new(),
        10000,
        baseline_paths,
    );

    let target = create_test_profile(
        "0xbbb",
        "1.0.0",
        250,
        5,
        HashMap::new(),
        10000,
        target_paths,
    );

    let diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    // Verify common path (func_a)
    assert_eq!(diff.deltas.hot_paths.common_paths.len(), 1);
    let common = &diff.deltas.hot_paths.common_paths[0];
    assert_eq!(common.stack, "main;func_a");
    assert_eq!(common.baseline_gas, 100);
    assert_eq!(common.target_gas, 150);
    assert_eq!(common.gas_change, 50);
    assert_eq!(common.percent_change, 50.0);

    // Verify disappeared path (func_b)
    assert_eq!(diff.deltas.hot_paths.baseline_only.len(), 1);
    assert_eq!(diff.deltas.hot_paths.baseline_only[0].stack, "main;func_b");

    // Verify new path (func_c)
    assert_eq!(diff.deltas.hot_paths.target_only.len(), 1);
    assert_eq!(diff.deltas.hot_paths.target_only[0].stack, "main;func_c");
}

// ============================================================================
// Additional Edge Case Tests
// ============================================================================

#[test]
fn test_negative_gas_change() {
    let baseline = create_test_profile(
        "0xaaa",
        "1.0.0",
        200000,
        30,
        HashMap::new(),
        50000,
        vec![],
    );

    let target = create_test_profile(
        "0xbbb",
        "1.0.0",
        150000,
        25,
        HashMap::new(),
        40000,
        vec![],
    );

    let diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    assert_eq!(diff.deltas.gas.absolute_change, -50000);
    assert!(diff.deltas.gas.percent_change < 0.0);
}

#[test]
fn test_empty_hot_paths() {
    let baseline = create_test_profile(
        "0xaaa",
        "1.0.0",
        100000,
        10,
        HashMap::new(),
        20000,
        vec![],
    );

    let target = create_test_profile(
        "0xbbb",
        "1.0.0",
        110000,
        12,
        HashMap::new(),
        22000,
        vec![],
    );

    let diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    assert_eq!(diff.deltas.hot_paths.common_paths.len(), 0);
    assert_eq!(diff.deltas.hot_paths.baseline_only.len(), 0);
    assert_eq!(diff.deltas.hot_paths.target_only.len(), 0);
}

#[test]
fn test_threshold_absolute_gas() {
    let baseline = create_baseline_profile();
    let target = create_regression_profile();

    let mut diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    // Test absolute gas threshold
    let thresholds = ThresholdConfig {
        gas: GasThresholds {
            max_increase_percent: None,
            max_increase_absolute: Some(50000), // Actual is 60000
        },
        hostio: HostIOThresholds::default(),
        hot_paths: None,
    };

    let violations = check_thresholds(&mut diff, &thresholds);

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].metric, "gas.max_increase_absolute");
    assert_eq!(violations[0].threshold, 50000.0);
    assert_eq!(violations[0].actual, 60000.0);
}

#[test]
fn test_hostio_per_type_limits() {
    let baseline = create_baseline_profile();
    let target = create_regression_profile();

    let mut diff = generate_diff(&baseline, &target).expect("Diff generation failed");

    let mut limits = HashMap::new();
    limits.insert("storage_load_max_increase".to_string(), 5); // Actual is 12

    let thresholds = ThresholdConfig {
        gas: GasThresholds::default(),
        hostio: HostIOThresholds {
            max_total_calls_increase_percent: None,
            limits: Some(limits),
        },
        hot_paths: None,
    };

    let violations = check_thresholds(&mut diff, &thresholds);

    let storage_violation = violations
        .iter()
        .find(|v| v.metric.contains("storage_load"));
    assert!(storage_violation.is_some());
    assert_eq!(storage_violation.unwrap().threshold, 5.0);
    assert_eq!(storage_violation.unwrap().actual, 12.0);
}