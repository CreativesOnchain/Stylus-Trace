//! Aggregation of trace data into collapsed stacks and metrics.
//!
//! This module transforms parsed execution traces into:
//! - Collapsed stack format (for flamegraph generation)
//! - Hot path analysis (top gas consumers)
//! - Gas distribution statistics

pub mod stack_builder;
pub mod metrics;

// Re-export main types and functions
pub use stack_builder::{CollapsedStack, build_collapsed_stacks, merge_small_stacks};
pub use metrics::{calculate_hot_paths, calculate_gas_distribution, GasDistribution};