//! Aggregation of trace data into collapsed stacks and metrics.
//!
//! This module transforms parsed execution traces into:
//! - Collapsed stack format (for flamegraph generation)
//! - Hot path analysis (top gas consumers)
//! - Gas distribution statistics

pub mod metrics;
pub mod stack_builder;

// Re-export main types and functions
pub use metrics::{calculate_gas_distribution, calculate_hot_paths};
pub use stack_builder::build_collapsed_stacks;
