use std::path::PathBuf;
use crate::flamegraph::FlamegraphConfig;

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
