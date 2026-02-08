//! Output writers for profile data and flamegraphs.
//!
//! This module handles writing data to disk in various formats:
//! - JSON profiles (pretty and compact)
//! - SVG flamegraphs
//! - Text summaries

pub mod json;
pub mod svg;

// Re-export main functions
pub use json::{write_profile, read_profile};
pub use svg::write_svg;