//! Stylus Trace Studio CLI
//!
//! A performance profiling tool for Arbitrum Stylus transactions.
//! Generates flamegraphs and detailed profiles from transaction traces.

use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Env;
use std::path::PathBuf;

mod commands;

use commands::{execute_capture, validate_args, CaptureArgs};
use stylus_trace_studio::flamegraph::FlamegraphConfig;
use stylus_trace_studio::utils::config::SCHEMA_VERSION;

/// Stylus Trace Studio - Performance profiling for Arbitrum Stylus
#[derive(Parser, Debug)]
#[command(name = "stylus-trace")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Capture and profile a transaction
    Capture {
        /// RPC endpoint URL
        #[arg(short, long, default_value = "http://localhost:8547")]
        rpc: String,
        
        /// Transaction hash to profile
        #[arg(short, long)]
        tx: String,
        
        /// Output path for JSON profile
        #[arg(short, long, default_value = "profile.json")]
        output: PathBuf,
        
        /// Output path for SVG flamegraph (optional)
        #[arg(short, long)]
        flamegraph: Option<PathBuf>,
        
        /// Number of top hot paths to include
        #[arg(long, default_value = "20")]
        top_paths: usize,
        
        /// Flamegraph title
        #[arg(long)]
        title: Option<String>,
        
        
        /// Flamegraph width in pixels
        #[arg(long, default_value = "1200")]
        width: usize,
        
        /// Print text summary to stdout
        #[arg(long)]
        summary: bool,

        /// Use Stylus Ink units (scaled by 10,000)
        #[arg(long)]
        ink: bool,

        /// Path to WASM binary with debug symbols (for source-to-line mapping)
        #[arg(long)]
        wasm: Option<PathBuf>,

        /// Optional tracer name (defaults to "stylusTracer" if omitted)
        #[arg(long)]
        tracer: Option<String>,
    },
    
    /// Validate a profile JSON file
    Validate {
        /// Path to profile JSON file
        #[arg(short, long)]
        file: PathBuf,
    },
    
    /// Display schema information
    Schema {
        /// Show full schema details
        #[arg(long)]
        show: bool,
    },
    
    /// Display version information
    Version,
}

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Setup logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();
    
    // Execute command
    match cli.command {
        Commands::Capture {
            rpc,
            tx,
            output,
            flamegraph,
            top_paths,
            title,

            width,
            summary,
            ink,
            wasm,
            tracer,
        } => {
            
            // Create flamegraph config
            let fg_config = if flamegraph.is_some() {
                let mut config = FlamegraphConfig::new();
                
                if let Some(title_str) = title {
                    config = config.with_title(title_str);
                }
                
                config.width = width;
                
                Some(config)
            } else {
                None
            };
            
            // Create capture args
            let args = CaptureArgs {
                rpc_url: rpc,
                transaction_hash: tx,
                output_json: output,
                output_svg: flamegraph,
                top_paths,
                flamegraph_config: fg_config.map(|c| c.with_ink(ink)),
                print_summary: summary,
                tracer,
                ink,
                wasm,
            };
            
            // Validate args first
            validate_args(&args)?;
            
            // Execute capture
            execute_capture(args)?;
        }
        
        Commands::Validate { file } => {
            validate_profile_file(file)?;
        }
        
        Commands::Schema { show } => {
            display_schema(show);
        }
        
        Commands::Version => {
            display_version();
        }
    }
    
    Ok(())
}



/// Validate a profile JSON file
///
/// **Private** - internal command implementation
fn validate_profile_file(file_path: PathBuf) -> Result<()> {
    use stylus_trace_studio::output::read_profile;
    
    println!("Validating profile: {}", file_path.display());
    
    let profile = read_profile(&file_path)?;
    
    println!("✓ Valid profile JSON");
    println!("  Version: {}", profile.version);
    println!("  Transaction: {}", profile.transaction_hash);
    println!("  Total Gas: {}", profile.total_gas);
    println!("  HostIO Calls: {}", profile.hostio_summary.total_calls);
    println!("  Hot Paths: {}", profile.hot_paths.len());
    
    Ok(())
}

/// Display schema information
///
/// **Private** - internal command implementation
fn display_schema(show_details: bool) {
    println!("Stylus Trace Studio Profile Schema");
    println!("Current Version: {}", SCHEMA_VERSION);
    println!();
    
    if show_details {
        println!("Schema Structure:");
        println!("  version: string          - Schema version (e.g., '1.0.0')");
        println!("  transaction_hash: string - Transaction hash");
        println!("  total_gas: number        - Total gas used");
        println!("  hostio_summary: object   - HostIO event statistics");
        println!("    total_calls: number    - Total HostIO calls");
        println!("    by_type: object        - Breakdown by HostIO type");
        println!("    total_hostio_gas: number - Gas consumed by HostIO");
        println!("  hot_paths: array         - Top gas-consuming execution paths");
        println!("    stack: string          - Stack trace");
        println!("    gas: number            - Gas consumed");
        println!("    percentage: number     - Percentage of total gas");
        println!("    source_hint: object?   - Source location (if available)");
        println!("  generated_at: string     - ISO 8601 timestamp");
    } else {
        println!("Use --show for detailed schema information");
    }
}

/// Display version information
///
/// **Private** - internal command implementation
fn display_version() {
    println!("Stylus Trace Studio v{}", env!("CARGO_PKG_VERSION"));
    println!("Profile Schema: v{}", SCHEMA_VERSION);
    println!();
    println!("A performance profiling tool for Arbitrum Stylus transactions.");
    println!("https://github.com/your-org/stylus-trace-studio");
}