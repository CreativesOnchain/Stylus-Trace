//! SVG flamegraph output writer.
//!
//! Writes SVG content to files with proper encoding.

use crate::utils::error::OutputError;
use log::{debug, info};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Write SVG content to a file
///
/// **Public** - main entry point for SVG output
///
/// # Arguments
/// * `svg_content` - SVG string from flamegraph generator
/// * `output_path` - Path to output SVG file
///
/// # Returns
/// Ok if file written successfully
///
/// # Errors
/// * `OutputError::WriteFailed` - I/O error during write
/// * `OutputError::InvalidPath` - Path is invalid
///
/// # Example
/// ```ignore
/// let svg = generate_flamegraph(&stacks, None)?;
/// write_svg(&svg, "flamegraph.svg")?;
/// ```
pub fn write_svg(svg_content: &str, output_path: impl AsRef<Path>) -> Result<(), OutputError> {
    let output_path = output_path.as_ref();

    info!("Writing SVG to: {}", output_path.display());

    // Validate path
    super::validate_path(output_path)?;

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            debug!("Creating parent directories: {}", parent.display());
            std::fs::create_dir_all(parent)
                .map_err(|e| OutputError::InvalidPath(format!("Cannot create directory: {}", e)))?;
        }
    }

    // Open file for writing
    let file = File::create(output_path).map_err(OutputError::WriteFailed)?;

    let mut writer = BufWriter::new(file);

    // Write SVG content
    writer
        .write_all(svg_content.as_bytes())
        .map_err(OutputError::WriteFailed)?;

    writer.flush().map_err(OutputError::WriteFailed)?;

    let file_size = svg_content.len();
    info!(
        "SVG written successfully ({} bytes, {:.2} KB)",
        file_size,
        file_size as f64 / 1024.0
    );

    Ok(())
}
