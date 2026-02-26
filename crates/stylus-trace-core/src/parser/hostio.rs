//! HostIO event extraction and categorization.
//!
//! HostIO events represent calls from WASM to the Stylus VM runtime.
//! Common types: storage_read, storage_write, call, log, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of HostIO operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostIoType {
    StorageLoad,
    StorageStore,
    StorageFlush,
    StorageCache,
    Call,
    StaticCall,
    DelegateCall,
    Create,
    Log,
    SelfDestruct,
    AccountBalance,
    BlockHash,
    // Stylus specific
    NativeKeccak256,
    ReadArgs,
    WriteResult,
    MsgValue,
    MsgSender,
    MsgReentrant,
    Other,
}

impl std::str::FromStr for HostIoType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "storage_load" | "sload" | "storage_load_bytes32" => Self::StorageLoad,
            "storage_store" | "sstore" | "storage_store_bytes32" => Self::StorageStore,
            "storage_flush" | "storage_flush_cache" => Self::StorageFlush,
            "storage_cache" | "storage_cache_bytes32" => Self::StorageCache,
            "call" => Self::Call,
            "staticcall" => Self::StaticCall,
            "delegatecall" => Self::DelegateCall,
            "create" | "create2" => Self::Create,
            "log" | "log0" | "log1" | "log2" | "log3" | "log4" | "emit_log" => Self::Log,
            "selfdestruct" => Self::SelfDestruct,
            "balance" | "account_balance" => Self::AccountBalance,
            "blockhash" | "block_hash" => Self::BlockHash,
            "native_keccak256" | "keccak256" | "keccak" => Self::NativeKeccak256,
            "read_args" | "calldatacopy" | "memory_read" => Self::ReadArgs,
            "write_result" | "return" | "memory_write" => Self::WriteResult,
            "msg_value" | "callvalue" => Self::MsgValue,
            "msg_sender" | "caller" => Self::MsgSender,
            "msg_reentrant" => Self::MsgReentrant,
            _ => Self::Other,
        })
    }
}

impl HostIoType {
    /// Try to map an EVM opcode or instruction to a HostIO type
    pub fn from_opcode(op: &str) -> Option<Self> {
        match op.to_uppercase().as_str() {
            "SLOAD" => Some(Self::StorageLoad),
            "SSTORE" => Some(Self::StorageFlush), // In Stylus, SSTORE often means flush
            "LOG0" | "LOG1" | "LOG2" | "LOG3" | "LOG4" => Some(Self::Log),
            "CALL" => Some(Self::Call),
            "STATICCALL" => Some(Self::StaticCall),
            "DELEGATECALL" => Some(Self::DelegateCall),
            "CREATE" | "CREATE2" => Some(Self::Create),
            "SELFDESTRUCT" => Some(Self::SelfDestruct),
            "BALANCE" => Some(Self::AccountBalance),
            "BLOCKHASH" => Some(Self::BlockHash),
            _ => None,
        }
    }
}

/// A single HostIO event from the trace
#[derive(Debug, Clone)]
pub struct HostIoEvent {
    pub io_type: HostIoType,
    pub gas_cost: u64,
}

/// Aggregated HostIO statistics
#[derive(Debug, Clone)]
pub struct HostIoStats {
    counts: HashMap<HostIoType, u64>,
    total_gas: u64,
}

impl HostIoStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            total_gas: 0,
        }
    }

    /// Add a HostIO event to the statistics
    pub fn add_event(&mut self, event: HostIoEvent) {
        *self.counts.entry(event.io_type).or_insert(0) += 1;
        self.total_gas += event.gas_cost;
    }

    /// Get total number of HostIO calls
    pub fn total_calls(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Get count for a specific HostIO type
    pub fn count_for_type(&self, io_type: HostIoType) -> u64 {
        self.counts.get(&io_type).copied().unwrap_or(0)
    }

    /// Get total gas consumed by HostIO
    pub fn total_gas(&self) -> u64 {
        self.total_gas
    }

    /// Convert to a map for JSON serialization
    pub fn to_map(&self) -> HashMap<String, u64> {
        self.counts
            .iter()
            .map(|(k, v)| {
                let name = match k {
                    HostIoType::StorageLoad => "storage_load",
                    HostIoType::StorageStore => "storage_store",
                    HostIoType::StorageFlush => "storage_flush_cache",
                    HostIoType::StorageCache => "storage_cache",
                    HostIoType::Call => "call",
                    HostIoType::StaticCall => "staticcall",
                    HostIoType::DelegateCall => "delegatecall",
                    HostIoType::Create => "create",
                    HostIoType::Log => "emit_log",
                    HostIoType::SelfDestruct => "selfdestruct",
                    HostIoType::AccountBalance => "account_balance",
                    HostIoType::BlockHash => "block_hash",
                    HostIoType::NativeKeccak256 => "native_keccak256",
                    HostIoType::ReadArgs => "read_args",
                    HostIoType::WriteResult => "write_result",
                    HostIoType::MsgValue => "msg_value",
                    HostIoType::MsgSender => "msg_sender",
                    HostIoType::MsgReentrant => "msg_reentrant",
                    HostIoType::Other => "other",
                };
                (name.to_string(), *v)
            })
            .collect()
    }

    /// Convert to summary for inclusion in the final profile
    pub fn to_summary(&self) -> super::schema::HostIoSummary {
        super::schema::HostIoSummary {
            total_calls: self.total_calls(),
            by_type: self.to_map(),
            total_hostio_gas: self.total_gas(),
        }
    }
}

impl Default for HostIoStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract HostIO events from raw trace data
///
/// **Public** - used by the main parser to build statistics
///
/// # Arguments
/// * `trace_data` - Raw JSON from stylusTracer
///
/// # Returns
/// Parsed HostIO statistics
pub fn extract_hostio_events(trace_data: &serde_json::Value) -> HostIoStats {
    let mut stats = HostIoStats::new();

    // Try to extract HostIO array from trace
    // Actual field name depends on stylusTracer output format
    // This is a placeholder - adjust based on real trace format
    if let Some(hostio_array) = trace_data.get("hostio").and_then(|v| v.as_array()) {
        for event_json in hostio_array {
            if let Some(event) = parse_hostio_event(event_json) {
                stats.add_event(event);
            }
        }
    }

    stats
}

/// Parse a single HostIO event from JSON
pub fn parse_hostio_event(event_json: &serde_json::Value) -> Option<HostIoEvent> {
    let io_type_str = event_json.get("type")?.as_str()?;
    let gas_cost = event_json.get("gas")?.as_u64()?;

    Some(HostIoEvent {
        io_type: io_type_str.parse().unwrap(),
        gas_cost,
    })
}
