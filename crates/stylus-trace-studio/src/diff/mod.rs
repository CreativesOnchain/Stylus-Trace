//! Profile diff generation and threshold checking.
//!
//! This module compares two Profile JSONs (baseline vs target) and produces
//! delta reports with threshold violation detection.
//!
//! # Example
//! ```ignore
//! use stylus_trace_studio::diff::{generate_diff, load_thresholds};
//! use stylus_trace_studio::output::json::read_profile;
//!
//! let baseline = read_profile("baseline.json")?;
//! let target = read_profile("target.json")?;
//! let diff = generate_diff(&baseline, &target)?;
//!
//! let thresholds = load_thresholds("thresholds.toml")?;
//! let violations = check_thresholds(&diff, &thresholds);
//! ```

mod analyzer;
mod engine;
mod normalizer;
mod output;
mod schema;
mod threshold;

// Public API exports
pub use analyzer::analyze_profile;
pub use engine::generate_diff;
pub use normalizer::{calculate_gas_delta, calculate_hostio_type_changes, safe_percentage};
pub use output::render_terminal_diff;
pub use schema::{
    Deltas, DiffReport, DiffSummary, GasDelta, HostIOTypeChange, HostIoDelta, HotPathComparison,
    HotPathsDelta, ProfileMetadata, ThresholdViolation,
};
pub use threshold::{
    check_gas_thresholds, check_thresholds, create_summary, load_thresholds, GasThresholds,
    HostIOThresholds, HotPathThresholds, ThresholdConfig,
};

pub use crate::utils::error::DiffError;
