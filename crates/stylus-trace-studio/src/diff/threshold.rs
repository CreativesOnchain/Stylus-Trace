//! Threshold configuration and violation detection.
//!
//! Loads threshold policies from TOML and checks diff reports
//! for violations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::schema::{DiffReport, DiffSummary, ThresholdViolation};
use super::DiffError;

/// Complete threshold configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThresholdConfig {
    /// Gas thresholds
    #[serde(default)]
    pub gas: GasThresholds,

    /// HostIO thresholds
    #[serde(default)]
    pub hostio: HostIOThresholds,

    /// Hot path thresholds (optional)
    #[serde(default)]
    pub hot_paths: Option<HotPathThresholds>,
}

/// Gas-related thresholds
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GasThresholds {
    /// Maximum allowed gas increase percentage
    pub max_increase_percent: Option<f64>,

    /// Maximum allowed absolute gas increase
    pub max_increase_absolute: Option<u64>,
}

/// HostIO-related thresholds
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HostIOThresholds {
    /// Maximum allowed percentage increase in total HostIO calls
    pub max_total_calls_increase_percent: Option<f64>,

    /// Per-type absolute limits (e.g., storage_load_max_increase: 5)
    pub limits: Option<HashMap<String, u64>>,
}

/// Hot path thresholds
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotPathThresholds {
    /// Warn if any single hot path increases by more than this percentage
    pub warn_individual_increase_percent: Option<f64>,
}

/// Load thresholds from a TOML file
///
/// # Arguments
/// * `path` - Path to the TOML configuration file
///
/// # Returns
/// Parsed ThresholdConfig
///
/// # Errors
/// * `DiffError::IoError` - If file cannot be read
/// * `DiffError::ThresholdParseFailed` - If TOML is invalid
///
/// # Example
/// ```ignore
/// let thresholds = load_thresholds("thresholds.toml")?;
/// ```
pub fn load_thresholds(path: impl AsRef<Path>) -> Result<ThresholdConfig, DiffError> {
    let contents = fs::read_to_string(path)?;
    let config: ThresholdConfig = toml::from_str(&contents)?;
    Ok(config)
}

/// Check a diff report against thresholds and update violations
///
/// # Arguments
/// * `diff` - Mutable reference to diff report to update
/// * `config` - Threshold configuration to check against
///
/// # Returns
/// Vector of violations (also updates diff.threshold_violations)
///
/// # Example
/// ```ignore
/// let mut diff = generate_diff(&baseline, &target)?;
/// let thresholds = load_thresholds("thresholds.toml")?;
/// check_thresholds(&mut diff, &thresholds);
/// ```
pub fn check_thresholds(
    diff: &mut DiffReport,
    config: &ThresholdConfig,
) -> Vec<ThresholdViolation> {
    let mut violations = Vec::new();

    // Check gas thresholds
    check_gas_thresholds(&diff.deltas.gas, &config.gas, &mut violations);

    // Check HostIO thresholds
    check_hostio_thresholds(&diff.deltas.hostio, &config.hostio, &mut violations);

    // Check hot path thresholds
    if let Some(hp_thresholds) = &config.hot_paths {
        check_hot_path_thresholds(&diff.deltas.hot_paths, hp_thresholds, &mut violations);
    }

    // Update diff report
    diff.threshold_violations = violations.clone();
    diff.summary = create_summary(&violations);

    violations
}

/// Check gas thresholds
fn check_gas_thresholds(
    gas_delta: &super::schema::GasDelta,
    thresholds: &GasThresholds,
    violations: &mut Vec<ThresholdViolation>,
) {
    // Check percentage increase
    if let Some(max_percent) = thresholds.max_increase_percent {
        if gas_delta.percent_change > max_percent {
            violations.push(ThresholdViolation {
                metric: "gas.max_increase_percent".to_string(),
                threshold: max_percent,
                actual: gas_delta.percent_change,
                severity: "error".to_string(),
            });
        }
    }

    // Check absolute increase
    if let Some(max_absolute) = thresholds.max_increase_absolute {
        if gas_delta.absolute_change > 0
            && gas_delta.absolute_change as u64 > max_absolute
        {
            violations.push(ThresholdViolation {
                metric: "gas.max_increase_absolute".to_string(),
                threshold: max_absolute as f64,
                actual: gas_delta.absolute_change as f64,
                severity: "error".to_string(),
            });
        }
    }
}

/// Check HostIO thresholds
fn check_hostio_thresholds(
    hostio_delta: &super::schema::HostIoDelta,
    thresholds: &HostIOThresholds,
    violations: &mut Vec<ThresholdViolation>,
) {
    // Check total calls percentage
    if let Some(max_percent) = thresholds.max_total_calls_increase_percent {
        if hostio_delta.total_calls_percent_change > max_percent {
            violations.push(ThresholdViolation {
                metric: "hostio.max_total_calls_increase_percent".to_string(),
                threshold: max_percent,
                actual: hostio_delta.total_calls_percent_change,
                severity: "error".to_string(),
            });
        }
    }

    // Check per-type limits
    if let Some(limits) = &thresholds.limits {
        for (hostio_type, max_increase) in limits {
            if let Some(change) = hostio_delta.by_type_changes.get(hostio_type) {
                if change.delta > 0 && change.delta as u64 > *max_increase {
                    violations.push(ThresholdViolation {
                        metric: format!("hostio.limits.{}_max_increase", hostio_type),
                        threshold: *max_increase as f64,
                        actual: change.delta as f64,
                        severity: "error".to_string(),
                    });
                }
            }
        }
    }
}

/// Check hot path thresholds
fn check_hot_path_thresholds(
    hot_paths_delta: &super::schema::HotPathsDelta,
    thresholds: &HotPathThresholds,
    violations: &mut Vec<ThresholdViolation>,
) {
    if let Some(max_percent) = thresholds.warn_individual_increase_percent {
        for comparison in &hot_paths_delta.common_paths {
            if comparison.percent_change > max_percent {
                violations.push(ThresholdViolation {
                    metric: format!("hot_paths.{}", comparison.stack),
                    threshold: max_percent,
                    actual: comparison.percent_change,
                    severity: "warning".to_string(),
                });
            }
        }
    }
}

/// Create summary based on violations
fn create_summary(violations: &[ThresholdViolation]) -> DiffSummary {
    let error_count = violations
        .iter()
        .filter(|v| v.severity == "error")
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| v.severity == "warning")
        .count();

    let status = if error_count > 0 {
        "FAILED"
    } else if warning_count > 0 {
        "WARNING"
    } else {
        "PASSED"
    };

    DiffSummary {
        has_regressions: error_count > 0,
        violation_count: violations.len(),
        status: status.to_string(),
        warning: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::schema::{Deltas, GasDelta, HostIoDelta, HotPathsDelta};

    #[test]
    fn test_gas_threshold_exceeded() {
        let gas_delta = GasDelta {
            baseline: 100,
            target: 150,
            absolute_change: 50,
            percent_change: 50.0,
        };

        let thresholds = GasThresholds {
            max_increase_percent: Some(10.0),
            max_increase_absolute: None,
        };

        let mut violations = Vec::new();
        check_gas_thresholds(&gas_delta, &thresholds, &mut violations);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].metric, "gas.max_increase_percent");
        assert_eq!(violations[0].threshold, 10.0);
        assert_eq!(violations[0].actual, 50.0);
    }

    #[test]
    fn test_gas_threshold_not_exceeded() {
        let gas_delta = GasDelta {
            baseline: 100,
            target: 105,
            absolute_change: 5,
            percent_change: 5.0,
        };

        let thresholds = GasThresholds {
            max_increase_percent: Some(10.0),
            max_increase_absolute: None,
        };

        let mut violations = Vec::new();
        check_gas_thresholds(&gas_delta, &thresholds, &mut violations);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_create_summary_with_errors() {
        let violations = vec![
            ThresholdViolation {
                metric: "test".to_string(),
                threshold: 10.0,
                actual: 20.0,
                severity: "error".to_string(),
            },
        ];

        let summary = create_summary(&violations);
        assert_eq!(summary.status, "FAILED");
        assert!(summary.has_regressions);
        assert_eq!(summary.violation_count, 1);
    }

    #[test]
    fn test_create_summary_with_warnings() {
        let violations = vec![
            ThresholdViolation {
                metric: "test".to_string(),
                threshold: 10.0,
                actual: 20.0,
                severity: "warning".to_string(),
            },
        ];

        let summary = create_summary(&violations);
        assert_eq!(summary.status, "WARNING");
        assert!(!summary.has_regressions);
        assert_eq!(summary.violation_count, 1);
    }
}