//! Stylus Trace Studio CLI
//!
//! A performance profiling tool for Arbitrum Stylus transactions.
//! Generates flamegraphs and detailed profiles from transaction traces.

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use env_logger::Env;
use log::info;
use std::path::PathBuf;

use stylus_trace_core::commands::{
    display_schema, display_version, execute_capture, validate_args, validate_profile_file,
    CaptureArgs,
};
use stylus_trace_core::flamegraph::FlamegraphConfig;
use stylus_trace_core::output::json::read_profile;
use stylus_trace_core::output::viewer::{generate_viewer, open_browser};

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

        /// Output path for JSON profile (placed in artifacts/capture/ by default)
        #[arg(short, long, default_value = "profile.json")]
        output: PathBuf,

        /// Output path for SVG flamegraph (placed in artifacts/capture/ by default)
        #[arg(short, long, default_missing_value = "flamegraph.svg", num_args = 0..=1)]
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

        /// Optional tracer name (defaults to "stylusTracer" if omitted)
        #[arg(long)]
        tracer: Option<String>,

        /// Path to baseline profile for on-the-fly diffing
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Simple increase threshold percentage (e.g., 5.0). Applies to Gas, HostIOs, and Hot Paths.
        #[arg(short = 'p', long = "threshold-percent")]
        threshold_percent: Option<f64>,

        /// Specific gas increase threshold percentage
        #[arg(long = "gas-threshold")]
        gas_threshold: Option<f64>,

        /// Specific HostIO calls increase threshold percentage
        #[arg(long = "hostio-threshold")]
        hostio_threshold: Option<f64>,

        /// Open interactive web viewer
        #[arg(long)]
        view: bool,
    },

    /// Compare two transaction profiles and detect regressions
    Diff(DiffSubArgs),

    /// Open a previously captured profile in the web viewer
    View {
        /// Transaction hash or path to profile JSON
        #[arg(short, long)]
        tx: String,

        /// RPC endpoint URL (optional, used if fetching new trace)
        #[arg(short, long, default_value = "http://localhost:8547")]
        rpc: String,
    },

    /// Validate a profile JSON file
    Validate {
        /// Path to profile JSON file
        #[arg(short, long)]
        file: PathBuf,
    },

    /// CI configuration and management
    Ci {
        #[command(subcommand)]
        subcommand: CiSubcommands,
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

    /// Simple increase threshold percentage (e.g., 5.0). Applies to Gas, HostIOs, and Hot Paths.
    #[arg(short = 'p', long = "threshold-percent")]
    pub threshold_percent: Option<f64>,

    /// Focus strictly on Gas regressions. Overrides TOML settings and suppresses alerts for HostIO/HotPaths.
    #[arg(long = "gas-threshold")]
    pub gas_threshold: Option<f64>,

    /// Focus strictly on HostIO regressions. Overrides TOML settings and suppresses alerts for Gas/HotPaths.
    #[arg(long = "hostio-threshold")]
    pub hostio_threshold: Option<f64>,

    /// Print a human-readable summary to the terminal
    #[arg(short, long, default_value_t = true)]
    pub summary: bool,

    /// Path to write the diff report JSON
    #[arg(short, long, default_value = "diff_report.json")]
    pub output: Option<PathBuf>,

    /// Path to write the visual diff flamegraph SVG
    #[arg(short = 'f', long, default_missing_value = "diff.svg", num_args = 0..=1)]
    pub flamegraph: Option<PathBuf>,

    /// Open interactive side-by-side web viewer
    #[arg(long)]
    pub view: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    setup_logging(cli.verbose);

    match cli.command {
        Commands::Capture { .. } => handle_capture(cli.command)?,
        Commands::Diff(ref args) => handle_diff(args)?,
        Commands::View { ref tx, ref rpc } => handle_view(tx, rpc)?,
        Commands::Validate { file } => {
            validate_profile_file(file).context("Failed to validate profile")?
        }
        Commands::Ci { subcommand } => handle_ci(subcommand)?,
        Commands::Schema { show } => display_schema(show),
        Commands::Version => display_version(),
    }

    Ok(())
}

#[derive(Subcommand, Debug)]
pub enum CiSubcommands {
    /// Initialize CI/CD performance regression checks
    Init {
        /// Transaction hash to profile in CI (optional)
        #[arg(short, long)]
        tx: Option<String>,

        /// RPC endpoint URL (optional)
        #[arg(short, long)]
        rpc: Option<String>,

        /// Global percentage threshold for all metrics (e.g., 1.0)
        #[arg(short = 'p', long, default_value = "1.0")]
        threshold: f64,

        /// Specific gas increase threshold percentage
        #[arg(long = "gas-threshold")]
        gas_threshold: Option<f64>,

        /// Specific HostIO calls increase threshold percentage
        #[arg(long = "hostio-threshold")]
        hostio_threshold: Option<f64>,

        /// Force overwrite existing workflow files
        #[arg(long)]
        force: bool,
    },
}

/// Handle CI command logic
fn handle_ci(subcommand: CiSubcommands) -> Result<()> {
    match subcommand {
        CiSubcommands::Init {
            tx,
            rpc,
            threshold,
            gas_threshold,
            hostio_threshold,
            force,
        } => {
            let args = stylus_trace_core::commands::models::CiInitArgs {
                transaction_hash: tx,
                rpc_url: rpc,
                threshold,
                gas_threshold,
                hostio_threshold,
                force,
            };
            stylus_trace_core::commands::execute_ci_init(args)
                .context("CI initialization failed")?;
        }
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
        tracer,
        baseline,
        threshold_percent,
        gas_threshold,
        hostio_threshold,
        view,
    } = command
    {
        // Enforce artifacts/ directory for relative paths
        output = resolve_artifact_path(output, "capture");

        if let Some(path) = flamegraph {
            flamegraph = Some(resolve_artifact_path(path, "capture"));
        }

        let baseline = baseline.map(|p| resolve_artifact_path(p, "capture"));

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
            baseline,
            threshold_percent,
            gas_threshold,
            hostio_threshold,
            wasm: None,
            view,
        };

        validate_args(&args).context("Invalid capture arguments")?;
        execute_capture(args).context("Capture execution failed")?;
    }

    Ok(())
}

/// Handle the diff command logic
fn handle_diff(args: &DiffSubArgs) -> Result<()> {
    let studio_args = stylus_trace_core::commands::models::DiffArgs {
        baseline: resolve_artifact_path(args.baseline.clone(), "capture"),
        target: resolve_artifact_path(args.target.clone(), "capture"),
        threshold_file: args.threshold.clone(),
        threshold_percent: args.threshold_percent,
        summary: args.summary,
        output: args
            .output
            .as_ref()
            .map(|p| resolve_artifact_path(p.clone(), "diff")),
        output_svg: args
            .flamegraph
            .as_ref()
            .map(|p| resolve_artifact_path(p.clone(), "diff")),
        gas_threshold: args.gas_threshold,
        hostio_threshold: args.hostio_threshold,
        view: args.view,
    };

    stylus_trace_core::commands::diff::execute_diff(studio_args)
        .context("Diff execution failed")?;
    Ok(())
}

/// Handle the view command logic
fn handle_view(tx_or_path: &str, rpc: &str) -> Result<()> {
    let path = PathBuf::from(tx_or_path);

    // Check if it's an existing JSON file
    if path.exists() && path.extension().is_some_and(|ext| ext == "json") {
        info!("Opening existing profile: {}", path.display());
        let profile = read_profile(&path).context("Failed to read profile JSON")?;
        let viewer_path = path.with_extension("html");
        generate_viewer(&profile, &viewer_path)?;
        open_browser(&viewer_path)?;
    } else if tx_or_path.starts_with("0x") && tx_or_path.len() == 66 {
        info!("Capturing and viewing transaction: {}", tx_or_path);
        let output = resolve_artifact_path(PathBuf::from("profile.json"), "capture");
        let args = CaptureArgs {
            rpc_url: rpc.to_string(),
            transaction_hash: tx_or_path.to_string(),
            output_json: output,
            view: true,
            ..Default::default()
        };
        execute_capture(args).context("Capture and view failed")?;
    } else {
        anyhow::bail!("Invalid input: provide a path to a .json profile or a 0x transaction hash");
    }

    Ok(())
}

/// Resolves a path to the artifacts/<category> directory if it's a simple filename
fn resolve_artifact_path(path: PathBuf, category: &str) -> PathBuf {
    if path
        .parent()
        .map(|p| p.as_os_str().is_empty())
        .unwrap_or(true)
    {
        PathBuf::from("artifacts").join(category).join(path)
    } else {
        path
    }
}
