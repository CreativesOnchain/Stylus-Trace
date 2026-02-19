//! Schema definitions for diff reports.
//!
//! Defines the structures that represent differences between two profiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete diff report comparing baseline and target profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    /// Schema version for the diff format
    pub diff_version: String,

    /// Timestamp when diff was generated
    pub generated_at: String,

    /// Metadata from baseline profile
    pub baseline: ProfileMetadata,

    /// Metadata from target profile
    pub target: ProfileMetadata,

    /// Calculated deltas between profiles
    pub deltas: Deltas,

    /// List of threshold violations (if any)
    pub threshold_violations: Vec<ThresholdViolation>,

    /// Analysis insights (Option 4: Heuristics)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub insights: Vec<AnalysisInsight>,

    /// Summary of diff results
    pub summary: DiffSummary,
}

/// Metadata extracted from a profile for comparison
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileMetadata {
    /// Transaction hash
    pub transaction_hash: String,

    /// Total gas used
    pub total_gas: u64,

    /// When the profile was generated
    pub generated_at: String,
}

/// All calculated deltas
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Deltas {
    /// Gas usage changes
    pub gas: GasDelta,

    /// HostIO changes
    pub hostio: HostIoDelta,

    /// Hot path changes
    pub hot_paths: HotPathsDelta,
}

/// Gas usage delta
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GasDelta {
    /// Baseline gas
    pub baseline: u64,

    /// Target gas
    pub target: u64,

    /// Absolute change (can be negative)
    pub absolute_change: i64,

    /// Percentage change (can be negative)
    pub percent_change: f64,
}

/// HostIO statistics delta
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostIoDelta {
    /// Baseline total calls
    pub baseline_total_calls: u64,

    /// Target total calls
    pub target_total_calls: u64,

    /// Change in total calls
    pub total_calls_change: i64,

    /// Percentage change in total calls
    pub total_calls_percent_change: f64,

    /// Changes by HostIO type
    pub by_type_changes: HashMap<String, HostIOTypeChange>,

    /// Baseline total HostIO gas
    pub baseline_total_gas: u64,

    /// Target total HostIO gas
    pub target_total_gas: u64,

    /// Change in HostIO gas
    pub gas_change: i64,

    /// Percentage change in HostIO gas
    pub gas_percent_change: f64,
}

/// Change in a specific HostIO type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostIOTypeChange {
    /// Count in baseline
    pub baseline: u64,

    /// Count in target
    pub target: u64,

    /// Delta (target - baseline)
    pub delta: i64,
}

/// Hot paths comparison
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotPathsDelta {
    /// Paths present in both profiles
    pub common_paths: Vec<HotPathComparison>,

    /// Paths only in baseline (disappeared)
    pub baseline_only: Vec<crate::parser::schema::HotPath>,

    /// Paths only in target (new)
    pub target_only: Vec<crate::parser::schema::HotPath>,
}

/// Comparison of a single hot path present in both profiles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotPathComparison {
    /// Stack trace identifier
    pub stack: String,

    /// Gas in baseline
    pub baseline_gas: u64,

    /// Gas in target
    pub target_gas: u64,

    /// Change in gas
    pub gas_change: i64,

    /// Percentage change
    pub percent_change: f64,
}

/// A single threshold violation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThresholdViolation {
    /// Name of the metric that violated threshold
    pub metric: String,

    /// Threshold value
    pub threshold: f64,

    /// Actual value
    pub actual: f64,

    /// Severity: "error" or "warning"
    pub severity: String,
}

/// Summary of diff results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    /// Whether there are any regressions
    pub has_regressions: bool,

    /// Number of threshold violations
    pub violation_count: usize,

    /// Overall status: "PASSED", "FAILED", "WARNING"
    pub status: String,

    /// Optional warning message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// A qualitative insight from the trace analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisInsight {
    /// Category of the insight (e.g., "Storage", "HostIO")
    pub category: String,

    /// Human-readable description
    pub description: String,

    /// Severity of the insight
    pub severity: InsightSeverity,

    /// Optional tag for grouping
    pub tag: Option<String>,
}

/// Severity level for analysis insights
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InsightSeverity {
    /// Purely informational
    Info,
    /// Suggests a possible optimization
    Low,
    /// Significant optimization opportunity or suspicious pattern
    Medium,
    /// Urgent performance issue
    High,
}
