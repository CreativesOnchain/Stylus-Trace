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

mod engine;
mod normalizer;
mod schema;
mod threshold;

// Public API exports
pub use engine::generate_diff;
pub use schema::{
    DiffReport, DiffSummary, Deltas, GasDelta, HostIoDelta, HostIOTypeChange, HotPathComparison,
    HotPathsDelta, ProfileMetadata, ThresholdViolation,
};
pub use threshold::{
    check_thresholds, load_thresholds, GasThresholds, HostIOThresholds, HotPathThresholds,
    ThresholdConfig,
};

// Error type
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("Incompatible schema versions: baseline={0}, target={1}")]
    IncompatibleVersions(String, String),

    #[error("Failed to read profile: {0}")]
    ReadFailed(#[from] crate::utils::error::OutputError),

    #[error("Invalid threshold configuration: {0}")]
    InvalidThresholds(String),

    #[error("Threshold TOML parse error: {0}")]
    ThresholdParseFailed(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests;