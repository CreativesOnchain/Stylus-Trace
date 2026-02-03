//! HTTP client for communicating with Arbitrum Nitro node RPC endpoint.

use super::types::{JsonRpcResponse, RawTraceData};
use crate::utils::error::RpcError;
use crate::utils::config::DEFAULT_RPC_TIMEOUT;
use log::{debug, info};
use reqwest::blocking::Client;

/// RPC client for fetching trace data from Nitro node
pub struct RpcClient {
    client: Client,
    rpc_url: String,
}

impl RpcClient {
    /// Create a new RPC client
    pub fn new(rpc_url: impl Into<String>) -> Result<Self, RpcError> {
        let client = Client::builder()
            .timeout(DEFAULT_RPC_TIMEOUT)
            .build()
            .map_err(RpcError::RequestFailed)?;
        
        Ok(Self {
            client,
            rpc_url: rpc_url.into(),
        })
    }

    // /// Create a client with custom timeout
/*
    pub fn with_timeout(
        rpc_url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, RpcError> {
        // ...
    }
*/

/*
    pub fn debug_trace_transaction(&self, tx_hash: &str) -> Result<RawTraceData, RpcError> {
        self.debug_trace_transaction_with_tracer(tx_hash, None)
    }
*/
    
    /// Fetch trace with optional tracer
    pub fn debug_trace_transaction_with_tracer(
        &self,
        tx_hash: &str,
        tracer: Option<&str>,
    ) -> Result<RawTraceData, RpcError> {
        let tx_hash = normalize_tx_hash(tx_hash);
        
        info!("Fetching trace for transaction: {}", tx_hash);
        
        // Build params based on tracer (defaulting to stylusTracer)
        let mut params_obj = serde_json::Map::new();
        params_obj.insert(
            "tracer".to_string(), 
            serde_json::json!(tracer.unwrap_or("stylusTracer"))
        );
        
        let params = serde_json::json!([
            tx_hash,
            params_obj
        ]);
        
        // Build RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "debug_traceTransaction",
            "params": params,
            "id": 1
        });
        
        debug!("RPC request: {:?}", request);
        
        // Make HTTP POST request
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .map_err(RpcError::RequestFailed)?;
        
        // Check HTTP status
        if !response.status().is_success() {
            return Err(RpcError::InvalidResponse(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }
        
        // Parse JSON-RPC response
        let rpc_response: JsonRpcResponse<RawTraceData> = response
            .json()
            .map_err(RpcError::RequestFailed)?;
        
        // Handle JSON-RPC error
        if let Some(error) = rpc_response.error {
            return Err(map_rpc_error(error, &tx_hash));
        }
        
        // Extract result
        rpc_response.result.ok_or_else(|| {
            RpcError::InvalidResponse("Missing result field".to_string())
        })
    }
}

/// Normalize transaction hash to include 0x prefix
fn normalize_tx_hash(tx_hash: &str) -> String {
    if tx_hash.starts_with("0x") {
        tx_hash.to_string()
    } else {
        format!("0x{}", tx_hash)
    }
}

/// Map JSON-RPC error to our error type
fn map_rpc_error(error: super::types::JsonRpcError, tx_hash: &str) -> RpcError {
    match error.code {
        -32000 => {
            if error.message.to_lowercase().contains("not found") {
                RpcError::TransactionNotFound(tx_hash.to_string())
            } else {
                RpcError::InvalidResponse(error.message)
            }
        }
        -32601 => {
            RpcError::TracerNotSupported
        }
        _ => RpcError::InvalidResponse(format!("{}: {}", error.code, error.message)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tx_hash() {
        assert_eq!(normalize_tx_hash("abc123"), "0xabc123");
        assert_eq!(normalize_tx_hash("0xdef456"), "0xdef456");
    }
}