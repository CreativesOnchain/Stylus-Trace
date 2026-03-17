//! Calculate performance metrics and hot paths from trace data.
//!
//! Hot paths are the execution paths that consume the most gas.
//! These are the primary targets for optimization.

use super::stack_builder::CollapsedStack;
use crate::parser::schema::{GasCategory, HotPath};
use log::debug;

/// Calculate hot paths from collapsed stacks
///
/// **Public** - main entry point for metrics calculation
///
/// # Arguments
/// * `stacks` - Collapsed stacks from stack_builder
/// * `total_gas` - Total gas used by transaction
/// * `top_n` - Number of top paths to return (e.g., 10)
///
/// # Returns
/// Vector of hot paths, sorted by gas consumption (descending)
pub fn calculate_hot_paths(
    stacks: &[CollapsedStack],
    _total_gas: u64,
    top_n: usize,
) -> Vec<HotPath> {
    debug!(
        "Calculating top {} hot paths from {} stacks",
        top_n,
        stacks.len()
    );

    // Total weight of these stacks is our base for percentages
    let execution_total: u64 = stacks.iter().map(|s| s.weight).sum();

    stacks
        .iter()
        .take(top_n)
        .map(|stack| create_hot_path(stack, execution_total))
        .collect()
}

/// Create a HotPath from a CollapsedStack
///
pub fn create_hot_path(stack: &CollapsedStack, denominator: u64) -> HotPath {
    // Calculate percentage based on passed denominator (usually total execution gas)
    let percentage = if denominator > 0 {
        (stack.weight as f64 / denominator as f64) * 100.0
    } else {
        0.0
    };

    // Determine the category from the leaf node of the stack.
    // E.g., for "root;call;storage_flush_cache" the leaf is "storage_flush_cache".
    let leaf = stack.stack.split(';').next_back().unwrap_or(&stack.stack);
    let category = categorize_stack_leaf(leaf);

    HotPath {
        stack: stack.stack.clone(),
        gas: stack.weight,
        percentage,
        category,
        source_hint: stack.last_pc.map(|pc| crate::parser::schema::SourceHint {
            file: "unknown".to_string(),
            line: None,
            column: None,
            function: Some(format!("0x{:x}", pc)), // Temporary: store PC in function field
        }),
    }
}

/// Categorize a hot-path leaf node into a [`GasCategory`].
///
/// This mirrors (and supersedes) the frontend `getCategory()` heuristic in
/// `viewer.js`, but is authoritative because it lives in the Rust backend
/// where `HostIoType` semantics are well-defined.
pub fn categorize_stack_leaf(leaf: &str) -> GasCategory {
    let n = leaf.to_lowercase();

    if n == "root" {
        return GasCategory::Root;
    }
    // Expensive storage writes
    if n.contains("storage_store") || n.contains("storage_flush") {
        return GasCategory::StorageExpensive;
    }
    // Cheaper storage reads
    if n.contains("storage_load") || n.contains("storage_cache") {
        return GasCategory::StorageNormal;
    }
    // Cryptographic ops
    if n.contains("keccak") {
        return GasCategory::Crypto;
    }
    // Memory / ABI
    if n.contains("memory") || n == "read_args" || n == "write_result" {
        return GasCategory::Memory;
    }
    // External calls / creates
    if n == "call" || n == "staticcall" || n == "delegatecall" || n.contains("create") {
        return GasCategory::Call;
    }
    // System / context
    if n.contains("msg_") || n.contains("block_") || n == "account_balance" {
        return GasCategory::System;
    }
    // Catch-all: user code
    GasCategory::UserCode
}

/// Calculate gas distribution statistics
///
/// **Public** - provides summary statistics
///
/// # Arguments
/// * `stacks` - Collapsed stacks
///
/// # Returns
/// Statistics about gas distribution
pub fn calculate_gas_distribution(stacks: &[CollapsedStack]) -> GasDistribution {
    if stacks.is_empty() {
        return GasDistribution::default();
    }

    let total: u64 = stacks.iter().map(|s| s.weight).sum();
    let count = stacks.len();
    let mean = total / count.max(1) as u64;

    // Get median
    let mut weights: Vec<u64> = stacks.iter().map(|s| s.weight).collect();
    weights.sort_unstable();
    let median = if weights.is_empty() {
        0
    } else {
        weights[weights.len() / 2]
    };

    // Top 10% of stacks
    let top_10_percent_count = (count as f64 * 0.1).ceil() as usize;
    let top_10_percent_gas: u64 = stacks
        .iter()
        .take(top_10_percent_count)
        .map(|s| s.weight)
        .sum();

    GasDistribution {
        total_gas: total,
        stack_count: count,
        mean_gas_per_stack: mean,
        median_gas_per_stack: median,
        top_10_percent_percentage: if total > 0 {
            (top_10_percent_gas as f64 / total as f64) * 100.0
        } else {
            0.0
        },
    }
}

/// Gas distribution statistics
///
/// **Public** - returned from calculate_gas_distribution
#[derive(Debug, Clone)]
pub struct GasDistribution {
    /// Total gas across all stacks
    pub total_gas: u64,

    /// Number of unique stacks
    pub stack_count: usize,

    /// Mean gas per stack
    pub mean_gas_per_stack: u64,

    /// Median gas per stack
    pub median_gas_per_stack: u64,

    /// Gas consumed by top 10% of stacks

    /// Percentage of total gas in top 10%
    pub top_10_percent_percentage: f64,
}

impl Default for GasDistribution {
    fn default() -> Self {
        Self {
            total_gas: 0,
            stack_count: 0,
            mean_gas_per_stack: 0,
            median_gas_per_stack: 0,
            top_10_percent_percentage: 0.0,
        }
    }
}

impl GasDistribution {
    /// Get human-readable summary
    ///
    /// **Public** - for logging and debugging
    pub fn summary(&self) -> String {
        format!(
            "Total: {} | Stacks: {} | Mean: {} | Median: {} | Top 10%: {:.1}%",
            self.total_gas,
            self.stack_count,
            self.mean_gas_per_stack,
            self.median_gas_per_stack,
            self.top_10_percent_percentage
        )
    }
}
