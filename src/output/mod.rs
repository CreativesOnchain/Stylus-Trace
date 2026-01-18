//! Output writers for profile data and flamegraphs.
//!
//! This module handles writing data to disk in various formats:
//! - JSON profiles (pretty and compact)
//! - SVG flamegraphs
//! - Text summaries

pub mod json;
pub mod svg;

// Re-export main functions
pub use json::{write_profile, write_profile_compact, read_profile, profile_to_string};
pub use svg::{write_svg, write_svg_validated, read_svg, get_svg_info, SvgInfo};