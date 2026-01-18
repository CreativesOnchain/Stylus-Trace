//! Utility modules for configuration, error handling, and logging.

pub mod error;
pub mod config;

// Re-export commonly used error types for convenience
pub use error::{RpcError, ParseError, FlamegraphError, OutputError};