//! Stylus Trace Studio
//!
//! Performance profiling and flamegraph generation for
//! Arbitrum Stylus transactions.
//!
//! This crate provides the core implementation for the
//! `stylus-trace` CLI tool.
//!
//! ## Getting Started
//!
//! Most users should install and use the CLI:
//!
//! ```bash
//! cargo install stylus-trace-studio
//! stylus-trace --help
//! ```
//!
//! For full documentation and examples, see:
//! https://github.com/CreativesOnchain/Stylus-Trace

mod aggregator;
mod commands;
mod flamegraph;
mod output;
mod parser;
mod rpc;
mod utils;
