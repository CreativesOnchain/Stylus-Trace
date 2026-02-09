use anyhow::Result;
use std::path::PathBuf;
use crate::output::read_profile;
use crate::utils::config::SCHEMA_VERSION;

/// Validate a profile JSON file
pub fn validate_profile_file(file_path: PathBuf) -> Result<()> {
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
pub fn display_schema(show_details: bool) {
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
pub fn display_version() {
    println!("Stylus Trace Studio v{}", env!("CARGO_PKG_VERSION"));
    println!("Profile Schema: v{}", SCHEMA_VERSION);
    println!();
    println!("A performance profiling tool for Arbitrum Stylus transactions.");
    println!("https://github.com/your-org/stylus-trace-studio");
}
