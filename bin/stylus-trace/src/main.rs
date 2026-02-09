//! Stylus Trace Studio CLI
//!
//! A performance profiling tool for Arbitrum Stylus transactions.
//! Generates flamegraphs and detailed profiles from transaction traces.

use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Env;
use std::path::PathBuf;

use stylus_trace_studio::commands::{
    display_schema, display_version, execute_capture, validate_args, validate_profile_file,
    CaptureArgs,
};
use stylus_trace_studio::flamegraph::FlamegraphConfig;

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

        /// Output path for JSON profile (placed in artifacts/ by default)
        #[arg(short, long, default_value = "artifacts/profile.json")]
        output: PathBuf,

        /// Output path for SVG flamegraph (placed in artifacts/ by default)
        #[arg(short, long, default_missing_value = "artifacts/flamegraph.svg", num_args = 0..=1)]
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
            mut output,
            mut flamegraph,
            top_paths,
            title,
            width,
            summary,
            ink,
            wasm,
            tracer,
        } => {
            // Ensure outputs go to artifacts/ if no directory is specified
            let artifacts_dir = PathBuf::from("artifacts");

            if output.parent().map(|p| p.as_os_str().is_empty()).unwrap_or(true) {
                output = artifacts_dir.join(output);
            }

            if let Some(ref mut fg) = flamegraph {
                if fg.parent().map(|p| p.as_os_str().is_empty()).unwrap_or(true) {
                    *fg = artifacts_dir.join(&fg);
                }
            }

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
