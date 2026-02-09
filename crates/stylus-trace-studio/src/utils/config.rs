//! Configuration and constants for the CLI.

use std::time::Duration;

/// Default timeout for RPC requests
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);

/// Current output schema version
pub const SCHEMA_VERSION: &str = "1.0.0";

// Constants for gas/ink conversion
// Stylus uses "Ink" as the unit, which is 10,000x smaller than EVM gas
// 1 gas = 10,000 ink
pub const GAS_TO_INK_MULTIPLIER: u64 = 10_000;
pub const MAX_REASONABLE_GAS: u64 = 100_000_000; // 100M gas limit

// Field names for trace parsing (different RPC implementations use different names)
pub const GAS_FIELD_NAMES: &[&str] = &["gas", "gasUsed", "gas_used", "totalGas", "total_gas"];
pub const STEP_FIELD_NAMES: &[&str] = &[
    "structLogs",
    "struct_logs",
    "steps",
    "trace",
    "result",
    "logs",
];
