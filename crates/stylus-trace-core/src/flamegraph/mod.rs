//! Flamegraph generation using the inferno library.
//!
//! This module converts collapsed stacks into interactive SVG flamegraphs.
//! Flamegraphs provide a visual representation of where gas is consumed.

pub mod diff_generator;
pub mod generator;

// Re-export main types
pub use diff_generator::generate_diff_flamegraph;
pub use generator::{generate_flamegraph, generate_text_summary, FlamegraphConfig};
