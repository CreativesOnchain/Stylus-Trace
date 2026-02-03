//! Capture command implementation.
//!
//! The capture command:
//! 1. Fetches trace data from RPC
//! 2. Parses the trace
//! 3. Builds collapsed stacks
//! 4. Generates flamegraph
//! 5. Calculates metrics
//! 6. Writes output files

use stylus_trace_studio::aggregator::{build_collapsed_stacks, calculate_hot_paths, calculate_gas_distribution};
use stylus_trace_studio::flamegraph::{generate_flamegraph, generate_text_summary, FlamegraphConfig};
use stylus_trace_studio::output::{write_profile, write_svg};
use stylus_trace_studio::parser::{parse_trace, to_profile, source_map::SourceMapper};
use stylus_trace_studio::rpc::RpcClient;
use anyhow::{Context, Result};
use log::{info, debug, warn};
use std::path::PathBuf;
use std::time::Instant;

/// Arguments for the capture command
///
/// **Public** - used by main.rs to construct from CLI args
#[derive(Debug, Clone)]
pub struct CaptureArgs {
    /// RPC endpoint URL
    pub rpc_url: String,
    
    /// Transaction hash to profile
    pub transaction_hash: String,
    
    /// Output path for JSON profile
    pub output_json: PathBuf,
    
    /// Output path for SVG flamegraph (optional)
    pub output_svg: Option<PathBuf>,
    
    /// Number of top hot paths to include in profile
    pub top_paths: usize,
    
    /// Flamegraph configuration
    pub flamegraph_config: Option<FlamegraphConfig>,
    
    /// Print text summary to stdout
    pub print_summary: bool,

    /// Optional tracer name (None = default opcode tracer)
    pub tracer: Option<String>,

    /// Show Stylus Ink units (scaled by 10,000)
    pub ink: bool,

    /// Path to WASM binary (optional)
    pub wasm: Option<PathBuf>,
}

impl Default for CaptureArgs {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: String::new(),
            output_json: PathBuf::from("profile.json"),
            output_svg: Some(PathBuf::from("flamegraph.svg")),
            top_paths: 20,
            flamegraph_config: None,
            print_summary: false,
            tracer: None,
            ink: false,
            wasm: None,
        }
    }
}

/// Execute the capture command
///
/// **Public** - main entry point called from main.rs
///
/// # Arguments
/// * `args` - Capture command arguments
///
/// # Returns
/// Ok if capture succeeds, Err with context if any step fails
///
/// # Errors
/// * RPC connection failures
/// * Trace parsing errors
/// * File write errors
///
/// # Example
/// ```ignore
/// let args = CaptureArgs {
///     rpc_url: "http://localhost:8547".to_string(),
///     transaction_hash: "0xabc123...".to_string(),
///     output_json: PathBuf::from("profile.json"),
///     output_svg: Some(PathBuf::from("flamegraph.svg")),
///     top_paths: 20,
///     flamegraph_config: None,
///     print_summary: true,
///     tracer: None,
/// };
/// 
/// execute_capture(args)?;
/// ```
pub fn execute_capture(args: CaptureArgs) -> Result<()> {
    let start_time = Instant::now();
    
    info!("Starting capture for transaction: {}", args.transaction_hash);
    info!("RPC endpoint: {}", args.rpc_url);
    
    // Step 1: Fetch trace from RPC
    info!("Step 1/6: Fetching trace from RPC...");
    let raw_trace = fetch_trace(&args.rpc_url, &args.transaction_hash, args.tracer.as_deref())
        .context("Failed to fetch trace from RPC")?;
    
    // Step 2: Parse trace
    info!("Step 2/6: Parsing trace data...");
    let parsed_trace = parse_trace(&args.transaction_hash, &raw_trace)
        .context("Failed to parse trace data")?;
    
    debug!("Parsed trace: {} gas used, {} execution steps",
           parsed_trace.total_gas_used,
           parsed_trace.execution_steps.len());

    // Initialize SourceMapper if WASM path is provided
    let mapper = if let Some(wasm_path) = &args.wasm {
        info!("Step 2.5/6: Loading WASM for source mapping: {}...", wasm_path.display());
        match SourceMapper::new(wasm_path) {
            Ok(m) => Some(m),
            Err(e) => {
                warn!("Failed to load WASM binary for source mapping: {}", e);
                warn!("Continuing without source mapping information.");
                None
            }
        }
    } else {
        None
    };
    
    // Step 3: Build collapsed stacks
    info!("Step 3/6: Building collapsed stacks...");
    let stacks = build_collapsed_stacks(&parsed_trace);
    
    debug!("Built {} unique stacks", stacks.len());
    
    // Calculate gas distribution statistics
    let gas_dist = calculate_gas_distribution(&stacks);
    info!("Gas distribution: {}", gas_dist.summary());
    
    // Step 4: Calculate hot paths (Percentages relative to Execution Total)
    info!("Step 4/6: Calculating top {} hot paths...", args.top_paths);
    let hot_paths = calculate_hot_paths(&stacks, 0, args.top_paths); // 0 is currently ignored since calculate_hot_paths sums internally
    
    // Step 5: Generate flamegraph (if requested)
    let svg_content = if args.output_svg.is_some() {
        info!("Step 5/6: Generating flamegraph...");
        let config = args.flamegraph_config.as_ref();
        let svg = generate_flamegraph(&stacks, config, mapper.as_ref())
            .context("Failed to generate flamegraph")?;
        Some(svg)
    } else {
        info!("Step 5/6: Skipping flamegraph generation (not requested)");
        None
    };
    
    // Step 6: Write outputs
    info!("Step 6/6: Writing output files...");
    
    // Create profile
    let profile = to_profile(&parsed_trace, hot_paths, mapper.as_ref());
    
    // Write JSON profile
    write_profile(&profile, &args.output_json)
        .context("Failed to write profile JSON")?;
    
    info!("✓ Profile written to: {}", args.output_json.display());
    
    // Write SVG flamegraph (if generated)
    if let (Some(svg), Some(svg_path)) = (svg_content, &args.output_svg) {
        write_svg(&svg, svg_path)
            .context("Failed to write flamegraph SVG")?;
        
        info!("✓ Flamegraph written to: {}", svg_path.display());
    }
    
    // Print text summary (if requested)
    if args.print_summary {
        let total_execution_gas: u64 = stacks.iter().map(|s| s.weight).sum();
        let intrinsic_gas = parsed_trace.total_gas_used.saturating_sub(total_execution_gas);
        
        let display_total = if args.ink { parsed_trace.total_gas_used } else { parsed_trace.total_gas_used / 10_000 };
        let display_exec = if args.ink { total_execution_gas } else { total_execution_gas / 10_000 };
        let display_intr = if args.ink { intrinsic_gas } else { intrinsic_gas / 10_000 };
        let unit = if args.ink { "ink" } else { "gas" };

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  📊 STYLUS TRANSACTION PROFILE SUMMARY");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Transaction: {}", args.transaction_hash);
        println!("  Total Gas:   {:>12} {}", display_total, unit);
        println!("  ├─ Execution:{:>12} {}", display_exec, unit);
        println!("  └─ Intrinsic:{:>12} {}", display_intr, unit);
        println!("  HostIO Calls: {}", parsed_trace.hostio_stats.total_calls());
        println!("  Unique Paths: {}", stacks.len());
        println!();
        println!("{}", generate_text_summary(&profile.hot_paths, 10, args.ink));
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
    
    let elapsed = start_time.elapsed();
    info!("Capture completed in {:.2}s", elapsed.as_secs_f64());
    
    Ok(())
}

/// Fetch trace from RPC endpoint
///
/// **Private** - internal helper for execute_capture
fn fetch_trace(rpc_url: &str, tx_hash: &str, tracer: Option<&str>) -> Result<serde_json::Value> {
    let client = RpcClient::new(rpc_url)
        .context("Failed to create RPC client")?;
    
    let trace = client.debug_trace_transaction_with_tracer(tx_hash, tracer)
        .context(format!("Failed to fetch trace for transaction {}", tx_hash))?;
    
    Ok(trace)
}

/// Validate capture arguments
///
/// **Public** - can be called before execute_capture for early validation
///
/// # Arguments
/// * `args` - Arguments to validate
///
/// # Returns
/// Ok if arguments are valid, Err with message if not
pub fn validate_args(args: &CaptureArgs) -> Result<()> {
    // Validate RPC URL
    if args.rpc_url.is_empty() {
        anyhow::bail!("RPC URL cannot be empty");
    }
    
    if !args.rpc_url.starts_with("http://") && !args.rpc_url.starts_with("https://") {
        anyhow::bail!("RPC URL must start with http:// or https://");
    }
    
    // Validate transaction hash
    if args.transaction_hash.is_empty() {
        anyhow::bail!("Transaction hash cannot be empty");
    }
    
    // Basic hex validation (with or without 0x prefix)
    let tx_hash = args.transaction_hash.strip_prefix("0x")
        .unwrap_or(&args.transaction_hash);
    
    if tx_hash.len() != 64 {
        anyhow::bail!("Transaction hash must be 32 bytes (64 hex characters)");
    }
    
    if !tx_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("Transaction hash contains invalid characters");
    }
    
    // Validate top_paths
    if args.top_paths == 0 {
        anyhow::bail!("top_paths must be greater than 0");
    }
    
    if args.top_paths > 1000 {
        anyhow::bail!("top_paths is too large (max 1000)");
    }
    
    Ok(())
}

// /// Quick capture with defaults (convenience function)
// ...
/*
pub fn quick_capture(rpc_url: &str, tx_hash: &str) -> Result<(PathBuf, PathBuf)> {
    let args = CaptureArgs {
        rpc_url: rpc_url.to_string(),
        transaction_hash: tx_hash.to_string(),
        output_json: PathBuf::from("profile.json"),
        output_svg: Some(PathBuf::from("flamegraph.svg")),
        top_paths: 20,
        flamegraph_config: None,
        print_summary: false,
        tracer: None,  // FIXED: Use default opcode tracer
    };
    
    execute_capture(args.clone())?;
    
    Ok((args.output_json, args.output_svg.unwrap()))
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_args_valid() {
        let args = CaptureArgs {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_empty_rpc() {
        let args = CaptureArgs {
            rpc_url: String::new(),
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_invalid_rpc_scheme() {
        let args = CaptureArgs {
            rpc_url: "ftp://localhost:8547".to_string(),
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_empty_tx_hash() {
        let args = CaptureArgs {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: String::new(),
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_short_tx_hash() {
        let args = CaptureArgs {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: "0x1234".to_string(),
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_invalid_hex() {
        let args = CaptureArgs {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: "0xGGGG567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_tx_hash_without_prefix() {
        let args = CaptureArgs {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_top_paths_zero() {
        let args = CaptureArgs {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            top_paths: 0,
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_top_paths_too_large() {
        let args = CaptureArgs {
            rpc_url: "http://localhost:8547".to_string(),
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            top_paths: 2000,
            ..Default::default()
        };
        
        assert!(validate_args(&args).is_err());
    }
}