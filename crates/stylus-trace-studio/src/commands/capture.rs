//! Capture command implementation.
//!
//! The capture command:
//! 1. Fetches trace data from RPC
//! 2. Parses the trace
//! 3. Builds collapsed stacks
//! 4. Generates flamegraph
//! 5. Calculates metrics
//! 6. Writes output files

use crate::aggregator::stack_builder::CollapsedStack;
use crate::aggregator::{build_collapsed_stacks, calculate_gas_distribution, calculate_hot_paths};
use crate::commands::models::{CaptureArgs, GasDisplay};
use crate::flamegraph::{generate_flamegraph, generate_text_summary};
use crate::output::{write_profile, write_svg};
use crate::parser::{
    parse_trace, schema::HotPath, source_map::SourceMapper, to_profile, ParsedTrace,
};
use crate::rpc::RpcClient;
use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::PathBuf;
use std::time::Instant;

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
///     ink: false,
///     wasm: None,
/// };
///
/// execute_capture(args)?;
/// ```
pub fn execute_capture(args: CaptureArgs) -> Result<()> {
    let start_time = Instant::now();

    info!(
        "Starting capture for transaction: {}",
        args.transaction_hash
    );
    info!("RPC endpoint: {}", args.rpc_url);

    info!("Fetching trace from RPC...");
    let raw_trace = fetch_trace(
        &args.rpc_url,
        &args.transaction_hash,
        args.tracer.as_deref(),
    )
    .context("Failed to fetch trace from RPC")?;

    info!("Parsing trace data...");
    let parsed_trace =
        parse_trace(&args.transaction_hash, &raw_trace).context("Failed to parse trace data")?;

    debug!(
        "Parsed trace: {} gas used, {} execution steps",
        parsed_trace.total_gas_used,
        parsed_trace.execution_steps.len()
    );

    let mapper = initialize_source_mapper(args.wasm.as_ref());

    info!("Building collapsed stacks...");
    let stacks = build_collapsed_stacks(&parsed_trace);
    debug!("Built {} unique stacks", stacks.len());

    let gas_dist = calculate_gas_distribution(&stacks);
    info!("Gas distribution: {}", gas_dist.summary());

    info!("Calculating top {} hot paths...", args.top_paths);
    let hot_paths = calculate_hot_paths(&stacks, 0, args.top_paths);

    let svg_content = if args.output_svg.is_some() {
        info!("Generating flamegraph...");
        let config = args.flamegraph_config.as_ref();
        Some(
            generate_flamegraph(&stacks, config, mapper.as_ref())
                .context("Failed to generate flamegraph")?,
        )
    } else {
        None
    };

    write_outputs(
        &args,
        &parsed_trace,
        hot_paths,
        mapper.as_ref(),
        svg_content,
    )?;

    if args.print_summary {
        print_transaction_summary(&args, &parsed_trace, &stacks, mapper.as_ref());
    }

    info!(
        "Capture completed in {:.2}s",
        start_time.elapsed().as_secs_f64()
    );
    Ok(())
}

/// Initialize SourceMapper if WASM path is provided.
///
/// **Private** - internal helper for execute_capture
fn initialize_source_mapper(wasm_path: Option<&PathBuf>) -> Option<SourceMapper> {
    let wasm_path = wasm_path?;
    info!(
        "Loading WASM for source mapping: {}...",
        wasm_path.display()
    );
    match SourceMapper::new(wasm_path) {
        Ok(m) => Some(m),
        Err(e) => {
            warn!("Failed to load WASM binary for source mapping: {}", e);
            warn!("Continuing without source mapping information.");
            None
        }
    }
}

/// Write output files (JSON profile and optional SVG flamegraph).
///
/// **Private** - internal helper for execute_capture
fn write_outputs(
    args: &CaptureArgs,
    parsed_trace: &ParsedTrace,
    hot_paths: Vec<HotPath>,
    mapper: Option<&SourceMapper>,
    svg_content: Option<String>,
) -> Result<()> {
    info!("Writing output files...");

    let profile = to_profile(parsed_trace, hot_paths, mapper);

    write_profile(&profile, &args.output_json).context("Failed to write profile JSON")?;
    info!("✓ Profile written to: {}", args.output_json.display());

    if let (Some(svg), Some(svg_path)) = (svg_content, &args.output_svg) {
        write_svg(&svg, svg_path).context("Failed to write flamegraph SVG")?;
        info!("✓ Flamegraph written to: {}", svg_path.display());
    }

    Ok(())
}

/// Print a human-readable transaction summary to stdout.
///
/// **Private** - internal helper for execute_capture
fn print_transaction_summary(
    args: &CaptureArgs,
    parsed_trace: &ParsedTrace,
    stacks: &[CollapsedStack],
    mapper: Option<&SourceMapper>,
) {
    let total_execution_gas: u64 = stacks.iter().map(|s| s.weight).sum();
    let intrinsic_gas = parsed_trace
        .total_gas_used
        .saturating_sub(total_execution_gas);

    let display = GasDisplay::new(args.ink);
    let profile = to_profile(
        parsed_trace,
        calculate_hot_paths(stacks, 0, args.top_paths),
        mapper,
    );

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  📊 STYLUS TRANSACTION PROFILE SUMMARY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Transaction: {}", args.transaction_hash);
    println!(
        "  Total Gas:   {:>12} {}",
        display.format(parsed_trace.total_gas_used),
        display.unit()
    );
    println!(
        "  ├─ Execution:{:>12} {}",
        display.format(total_execution_gas),
        display.unit()
    );
    println!(
        "  └─ Intrinsic:{:>12} {}",
        display.format(intrinsic_gas),
        display.unit()
    );
    println!(
        "  HostIO Calls: {}",
        parsed_trace.hostio_stats.total_calls()
    );
    println!("  Unique Paths: {}", stacks.len());
    println!();
    println!(
        "{}",
        generate_text_summary(&profile.hot_paths, 10, args.ink)
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Helper for formatting gas/ink units for display.
///
///**Private** - internal utility for print_transaction_summary
/// Fetch trace from RPC endpoint
///
/// **Private** - internal helper for execute_capture
fn fetch_trace(rpc_url: &str, tx_hash: &str, tracer: Option<&str>) -> Result<serde_json::Value> {
    let client = RpcClient::new(rpc_url).context("Failed to create RPC client")?;

    let trace = client
        .debug_trace_transaction_with_tracer(tx_hash, tracer)
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
    let tx_hash = args
        .transaction_hash
        .strip_prefix("0x")
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
