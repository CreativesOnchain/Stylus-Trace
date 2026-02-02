//! Configuration and constants for the CLI.

use std::time::Duration;

/// Default timeout for RPC requests
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);

// /// Maximum trace size we'll attempt to parse (10 MB)
/*
pub const MAX_TRACE_SIZE_BYTES: usize = 10 * 1024 * 1024;
*/

/// Current output schema version
pub const SCHEMA_VERSION: &str = "1.0.0";

// /// Configuration for the CLI (future extensibility)
/*
#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_timeout: Duration,
    pub max_trace_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_timeout: DEFAULT_RPC_TIMEOUT,
            max_trace_size: MAX_TRACE_SIZE_BYTES,
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }
}
*/