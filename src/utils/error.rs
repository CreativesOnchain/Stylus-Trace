//! Error types for the entire application.
//!
//! We use `thiserror` for library-style errors with custom types,
//! and `anyhow` for application-level error propagation in main.rs and commands.

use thiserror::Error;

/// Errors that can occur during RPC communication
#[derive(Error, Debug)]
pub enum RpcError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("Invalid RPC response: {0}")]
    InvalidResponse(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Tracer not supported by this RPC endpoint")]
    TracerNotSupported,
}

/// Errors that can occur during trace parsing
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("JSON deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid trace format: {0}")]
    InvalidFormat(String),
    
/*
    #[error("Unsupported schema version: {0}")]
    UnsupportedVersion(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
*/
}

/// Errors that can occur during flamegraph generation
#[derive(Error, Debug)]
pub enum FlamegraphError {
/*
    #[error("Failed to generate flamegraph: {0}")]
    GenerationFailed(String),
*/
    
    #[error("Empty stack data")]
    EmptyStacks,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Errors that can occur during file output
#[derive(Error, Debug)]
pub enum OutputError {
    #[error("Failed to write file: {0}")]
    WriteFailed(#[from] std::io::Error),
    
    #[error("Failed to serialize JSON: {0}")]
    SerializationFailed(#[from] serde_json::Error),
    
    #[error("Invalid output path: {0}")]
    InvalidPath(String),
}