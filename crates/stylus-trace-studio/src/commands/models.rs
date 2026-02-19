use crate::flamegraph::FlamegraphConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

    /// Path to baseline profile for on-the-fly diffing
    pub baseline: Option<std::path::PathBuf>,

    /// Simple gas increase threshold percentage for on-the-fly diffing
    pub threshold_percent: Option<f64>,

    /// Specific gas increase threshold percentage
    pub gas_threshold: Option<f64>,

    /// Specific HostIO calls increase threshold percentage
    pub hostio_threshold: Option<f64>,

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
            baseline: None,
            threshold_percent: None,
            gas_threshold: None,
            hostio_threshold: None,
        }
    }
}

pub struct GasDisplay {
    pub use_ink: bool,
}

impl GasDisplay {
    pub fn new(use_ink: bool) -> Self {
        Self { use_ink }
    }

    pub fn format(&self, gas: u64) -> u64 {
        if self.use_ink {
            gas
        } else {
            gas / 10_000
        }
    }

    pub fn unit(&self) -> &'static str {
        if self.use_ink {
            "ink"
        } else {
            "gas"
        }
    }
}
/// Arguments for the diff command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffArgs {
    /// Path to the baseline profile JSON
    pub baseline: PathBuf,

    /// Path to the target profile JSON
    pub target: PathBuf,

    /// Optional threshold configuration file (TOML)
    pub threshold_file: Option<PathBuf>,

    /// Simple gas increase threshold percentage (e.g., 5.0)
    pub threshold_percent: Option<f64>,

    /// Specific gas increase threshold percentage
    pub gas_threshold: Option<f64>,

    /// Specific HostIO calls increase threshold percentage
    pub hostio_threshold: Option<f64>,

    /// Print a human-readable summary to the terminal
    pub summary: bool,

    /// Path to write the diff report JSON
    pub output: Option<PathBuf>,

    /// Path to write the visual diff flamegraph SVG
    pub output_svg: Option<PathBuf>,
}

impl Default for DiffArgs {
    fn default() -> Self {
        Self {
            baseline: PathBuf::new(),
            target: PathBuf::new(),
            threshold_file: None,
            threshold_percent: None,
            gas_threshold: None,
            hostio_threshold: None,
            summary: true,
            output: None,
            output_svg: None,
        }
    }
}
