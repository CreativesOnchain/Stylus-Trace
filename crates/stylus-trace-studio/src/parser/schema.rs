//! Output JSON schema definitions for profile data.
//!
//! This module defines the structure of JSON files we write to disk.
//! Schema is versioned to allow future evolution.

use crate::aggregator::stack_builder::CollapsedStack;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level profile structure written to JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Schema version for compatibility checking
    pub version: String,

    /// Transaction hash that was profiled
    pub transaction_hash: String,

    /// Total gas used by the transaction
    pub total_gas: u64,

    /// Summary of HostIO events by category
    pub hostio_summary: HostIoSummary,

    /// Top hot paths (ranked by gas usage)
    pub hot_paths: Vec<HotPath>,

    /// Complete execution stacks (optional, for full diff visualization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_stacks: Option<Vec<CollapsedStack>>,

    /// Timestamp when profile was generated
    pub generated_at: String,
}

/// Summary statistics for HostIO events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostIoSummary {
    /// Total number of HostIO calls
    pub total_calls: u64,

    /// Breakdown by HostIO type
    pub by_type: HashMap<String, u64>,

    /// Total gas consumed by HostIO operations
    pub total_hostio_gas: u64,
}

/// A hot path in the execution (stack trace with gas)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotPath {
    /// Collapsed stack representation (e.g., "main;execute;storage_read")
    pub stack: String,

    /// Gas consumed by this path
    pub gas: u64,

    /// Percentage of total gas
    pub percentage: f64,

    /// Source hint (if debug symbols available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hint: Option<SourceHint>,
}

/// Source code location hint
///
/// NOTE: This is currently a placeholder/reserved feature. It is non-functional
/// because `stylusTracer` does not provide PC offsets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceHint {
    pub file: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub function: Option<String>,
}
