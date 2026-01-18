//! Types for JSON-RPC communication with Arbitrum Nitro node.
//!
//! Based on Ethereum JSON-RPC spec and Arbitrum's debug_traceTransaction extension.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request for debug_traceTransaction
    ///
    /// # Arguments
    /// * `tx_hash` - Transaction hash (with 0x prefix)
    /// * `id` - Request ID (for response correlation)
    pub fn debug_trace_transaction(tx_hash: String, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "debug_traceTransaction".to_string(),
            params: serde_json::json!([
                tx_hash,
                {
                    "tracer": "stylusTracer"
                }
            ]),
            id,
        }
    }
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(default)]
    pub result: Option<T>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object
#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Raw trace data from stylusTracer (opaque for now, parsed later)
///
/// We keep this as `serde_json::Value` because the exact schema
/// may vary between Nitro versions. The parser will handle validation.
pub type RawTraceData = serde_json::Value;