//! Flamegraph generation using the inferno library.
//!
//! This module converts collapsed stacks into interactive SVG flamegraphs.
//! Flamegraphs provide a visual representation of where gas is consumed.

pub mod generator;

// Re-export main types
pub use generator::{
    generate_flamegraph,
    generate_text_summary,
    FlamegraphConfig,
    FlamegraphPalette,
};