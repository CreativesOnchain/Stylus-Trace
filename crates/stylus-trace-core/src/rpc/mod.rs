//! RPC client for communicating with Arbitrum Nitro nodes.

pub mod client;
pub mod types;

// Re-export main types
pub use client::RpcClient;
