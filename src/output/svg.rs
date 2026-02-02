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
    validate_svg_path(output_path)?;
    
    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            debug!("Creating parent directories: {}", parent.display());
            std::fs::create_dir_all(parent)
                .map_err(|e| OutputError::InvalidPath(format!(
                    "Cannot create directory: {}",
                    e
                )))?;
        }
    }
    
    // Open file for writing
    let file = File::create(output_path)
        .map_err(OutputError::WriteFailed)?;
    
    let mut writer = BufWriter::new(file);
    
    // Write SVG content
    writer.write_all(svg_content.as_bytes())
        .map_err(OutputError::WriteFailed)?;
    
    writer.flush()
        .map_err(OutputError::WriteFailed)?;
    
    let file_size = svg_content.len();
    info!("SVG written successfully ({} bytes, {:.2} KB)", 
          file_size,
          file_size as f64 / 1024.0);
    
    Ok(())
}

/*
/// Validate SVG content before writing
/// ...
*/
/*
pub fn validate_svg_content(svg_content: &str) -> Result<(), OutputError> {
    // ...
}

pub fn write_svg_validated(
    svg_content: &str,
    output_path: impl AsRef<Path>,
) -> Result<(), OutputError> {
    // ...
}
*/

/// Validate output path for SVG
///
/// **Private** - internal validation
fn validate_svg_path(path: &Path) -> Result<(), OutputError> {
    // Check if path is empty
    if path.as_os_str().is_empty() {
        return Err(OutputError::InvalidPath("Path is empty".to_string()));
    }
    
    // Check if trying to overwrite a directory
    if path.exists() && path.is_dir() {
        return Err(OutputError::InvalidPath(format!(
            "Path is a directory: {}",
            path.display()
        )));
    }
    
    // Optionally check extension
    if let Some(ext) = path.extension() {
        if ext != "svg" {
            debug!("Warning: File does not have .svg extension: {}", path.display());
        }
    }
    
    Ok(())
}

// // /// Read SVG content from a file
// // ///
// // /// **Public** - useful for testing and validation
// // ///
// // /// # Arguments
// // /// * `input_path` - Path to SVG file
// // ///
// // /// # Returns
// // /// SVG content as string
/*
pub fn read_svg(input_path: impl AsRef<Path>) -> Result<String, OutputError> {
    // ...
}
*/

// /// Get SVG file metadata
// ///
// /// **Public** - useful for reporting file info
// ///
// /// # Arguments
// /// * `svg_path` - Path to SVG file
// ///
// /// # Returns
// /// Metadata about the SVG file
/*
pub fn get_svg_info(svg_path: impl AsRef<Path>) -> Result<SvgInfo, OutputError> {
    // ...
}
*/

// /// SVG file metadata
// ///
// /// **Public** - returned from get_svg_info
/*
#[derive(Debug, Clone)]
pub struct SvgInfo {
    // ...
}
*/

/*
impl SvgInfo {
    pub fn summary(&self) -> String {
        // ...
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    const VALID_SVG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect x="0" y="0" width="100" height="100" fill="red"/>
</svg>"#;

    #[test]
    fn test_write_and_read_svg() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        write_svg(VALID_SVG, path).unwrap();
        
        // let content = read_svg(path).unwrap();
        // assert_eq!(content, VALID_SVG);
    }

/*
    #[test]
    fn test_validate_svg_content_valid() { ... }
    ...
*/

    #[test]
    fn test_write_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested/dirs/flamegraph.svg");
        
        write_svg(VALID_SVG, &nested_path).unwrap();
        
        assert!(nested_path.exists());
    }

    #[test]
    fn test_validate_svg_path_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = validate_svg_path(temp_dir.path());
        assert!(result.is_err());
    }
}