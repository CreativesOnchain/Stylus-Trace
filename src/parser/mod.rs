//! Trace parsing and schema definitions.
//!
//! This module handles:
//! - Parsing raw JSON from stylusTracer
//! - Extracting HostIO events
//! - Validating trace format
//! - Defining output schema

pub mod hostio;
pub mod schema;
pub mod stylus_trace;

// Re-export main types
pub use hostio::HostIoType;
pub use stylus_trace::{parse_trace, to_profile, ParsedTrace};