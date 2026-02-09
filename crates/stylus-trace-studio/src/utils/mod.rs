//! Utility modules for configuration, error handling, and logging.

pub mod config;
pub mod error;

// Re-export commonly used error types for convenience
pub use error::FlamegraphError;
