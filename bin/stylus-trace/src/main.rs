//! Stylus Trace Studio CLI
//!
//! A performance profiling tool for Arbitrum Stylus transactions.
//! Generates flamegraphs and detailed profiles from transaction traces.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, Args};
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
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
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

        /// Path to baseline profile for on-the-fly diffing
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Simple gas increase threshold percentage for on-the-fly diffing
        #[arg(short = 'p', long)]
        threshold_percent: Option<f64>,
    },

    /// Compare two transaction profiles and detect regressions
    Diff(DiffSubArgs),

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

#[derive(Args, Debug)]
pub struct DiffSubArgs {
    /// Path to the baseline profile JSON
    pub baseline: PathBuf,

    /// Path to the target profile JSON
    pub target: PathBuf,

    /// Optional threshold configuration file (TOML)
    #[arg(short, long)]
    pub threshold: Option<PathBuf>,

    /// Simple gas increase threshold percentage (e.g., 5.0)
    #[arg(short = 'p', long)]
    pub threshold_percent: Option<f64>,

    /// Print a human-readable summary to the terminal
    #[arg(short, long, default_value_t = true)]
    pub summary: bool,

    /// Path to write the diff report JSON
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    setup_logging(cli.verbose);

    match cli.command {
        Commands::Capture { .. } => handle_capture(cli.command)?,
        Commands::Diff(ref args) => handle_diff(args)?,
        Commands::Validate { file } => {
            validate_profile_file(file).context("Failed to validate profile")?
        }
        Commands::Schema { show } => display_schema(show),
        Commands::Version => display_version(),
    }

    Ok(())
}

/// Setup logging based on verbosity level
fn setup_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();
}

/// Handle the capture command logic
fn handle_capture(command: Commands) -> Result<()> {
    if let Commands::Capture {
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
        baseline,
        threshold_percent,
    } = command
    {
        // Enforce artifacts/ directory for relative paths
        output = resolve_artifact_path(output);

        if let Some(path) = flamegraph {
            flamegraph = Some(resolve_artifact_path(path));
        }

        let baseline = baseline.map(resolve_artifact_path);

        // Build flamegraph configuration if requested
        let flamegraph_config = flamegraph.as_ref().map(|_| {
            let mut config = FlamegraphConfig::new().with_ink(ink);
            config.width = width;
            if let Some(t) = title {
                config = config.with_title(t);
            }
            config
        });

        let args = CaptureArgs {
            rpc_url: rpc,
            transaction_hash: tx,
            output_json: output,
            output_svg: flamegraph,
            top_paths,
            flamegraph_config,
            print_summary: summary,
            tracer,
            ink,
            wasm,
            baseline,
            threshold_percent,
        };

        validate_args(&args).context("Invalid capture arguments")?;
        execute_capture(args).context("Capture execution failed")?;
    }

    Ok(())
}

/// Handle the diff command logic
fn handle_diff(args: &DiffSubArgs) -> Result<()> {
    let studio_args = stylus_trace_studio::commands::models::DiffArgs {
        baseline: args.baseline.clone(),
        target: args.target.clone(),
        threshold_file: args.threshold.clone(),
        threshold_percent: args.threshold_percent,
        summary: args.summary,
        output: args.output.clone(),
    };

    stylus_trace_studio::commands::diff::execute_diff(studio_args)
        .context("Diff execution failed")?;
    Ok(())
}

/// Resolves a path to the artifacts directory if it's a simple filename
fn resolve_artifact_path(path: PathBuf) -> PathBuf {
    if path
        .parent()
        .map(|p| p.as_os_str().is_empty())
        .unwrap_or(true)
    {
        PathBuf::from("artifacts").join(path)
    } else {
        path
    }
}
