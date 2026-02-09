//! Output writers for profile data and flamegraphs.
//!
//! This module handles writing data to disk in various formats:
//! - JSON profiles (pretty and compact)
//! - SVG flamegraphs
//! - Text summaries

pub mod json;
pub mod svg;

// Re-export main functions
pub use json::{read_profile, write_profile};
pub use svg::write_svg;

use crate::utils::error::OutputError;
use std::path::Path;

/// Common path validation for output files
pub fn validate_path(path: &Path) -> Result<(), OutputError> {
    if path.as_os_str().is_empty() {
        return Err(OutputError::InvalidPath("Path is empty".to_string()));
    }

    if path.exists() && path.is_dir() {
        return Err(OutputError::InvalidPath(format!(
            "Path is a directory: {}",
            path.display()
        )));
    }

    Ok(())
}
