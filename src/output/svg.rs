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

/// Validate SVG content before writing
///
/// **Public** - useful for checking SVG validity before saving
///
/// # Arguments
/// * `svg_content` - SVG string to validate
///
/// # Returns
/// Ok if SVG appears valid, Err with details if not
pub fn validate_svg_content(svg_content: &str) -> Result<(), OutputError> {
    // Basic SVG validation
    if svg_content.is_empty() {
        return Err(OutputError::InvalidPath("SVG content is empty".to_string()));
    }
    
    // Check for SVG opening tag
    if !svg_content.contains("<svg") {
        return Err(OutputError::InvalidPath(
            "Content does not appear to be valid SVG (missing <svg tag)".to_string()
        ));
    }
    
    // Check for SVG closing tag
    if !svg_content.contains("</svg>") {
        return Err(OutputError::InvalidPath(
            "SVG content appears incomplete (missing </svg>)".to_string()
        ));
    }
    
    Ok(())
}

/// Write SVG with validation
///
/// **Public** - validates before writing
///
/// # Arguments
/// * `svg_content` - SVG string
/// * `output_path` - Path to output file
///
/// # Returns
/// Ok if valid and written successfully
pub fn write_svg_validated(
    svg_content: &str,
    output_path: impl AsRef<Path>,
) -> Result<(), OutputError> {
    // Validate content first
    validate_svg_content(svg_content)?;
    
    // Write if valid
    write_svg(svg_content, output_path)
}

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

/// Read SVG content from a file
///
/// **Public** - useful for testing and validation
///
/// # Arguments
/// * `input_path` - Path to SVG file
///
/// # Returns
/// SVG content as string
pub fn read_svg(input_path: impl AsRef<Path>) -> Result<String, OutputError> {
    let input_path = input_path.as_ref();
    
    debug!("Reading SVG from: {}", input_path.display());
    
    let content = std::fs::read_to_string(input_path)
        .map_err(OutputError::WriteFailed)?;
    
    Ok(content)
}

/// Get SVG file metadata
///
/// **Public** - useful for reporting file info
///
/// # Arguments
/// * `svg_path` - Path to SVG file
///
/// # Returns
/// Metadata about the SVG file
pub fn get_svg_info(svg_path: impl AsRef<Path>) -> Result<SvgInfo, OutputError> {
    let svg_path = svg_path.as_ref();
    
    let content = read_svg(svg_path)?;
    
    Ok(SvgInfo {
        path: svg_path.to_path_buf(),
        size_bytes: content.len(),
        size_kb: content.len() as f64 / 1024.0,
        line_count: content.lines().count(),
        is_valid: validate_svg_content(&content).is_ok(),
    })
}

/// SVG file metadata
///
/// **Public** - returned from get_svg_info
#[derive(Debug, Clone)]
pub struct SvgInfo {
    pub path: std::path::PathBuf,
    pub size_bytes: usize,
    pub size_kb: f64,
    pub line_count: usize,
    pub is_valid: bool,
}

impl SvgInfo {
    /// Get human-readable summary
    ///
    /// **Public** - for display
    pub fn summary(&self) -> String {
        format!(
            "SVG: {} | Size: {:.2} KB | Lines: {} | Valid: {}",
            self.path.display(),
            self.size_kb,
            self.line_count,
            if self.is_valid { "✓" } else { "✗" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    const VALID_SVG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect x="0" y="0" width="100" height="100" fill="red"/>
</svg>"#;

    const INVALID_SVG: &str = r#"<svg>
  <rect x="0" y="0"/>"#;  // Missing closing tag

    #[test]
    fn test_write_and_read_svg() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        write_svg(VALID_SVG, path).unwrap();
        
        let content = read_svg(path).unwrap();
        assert_eq!(content, VALID_SVG);
    }

    #[test]
    fn test_validate_svg_content_valid() {
        let result = validate_svg_content(VALID_SVG);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_svg_content_invalid() {
        let result = validate_svg_content(INVALID_SVG);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_svg_content_empty() {
        let result = validate_svg_content("");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_svg_validated() {
        let temp_file = NamedTempFile::new().unwrap();
        
        // Valid SVG should succeed
        let result = write_svg_validated(VALID_SVG, temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_svg_validated_invalid() {
        let temp_file = NamedTempFile::new().unwrap();
        
        // Invalid SVG should fail
        let result = write_svg_validated(INVALID_SVG, temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_svg_info() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        write_svg(VALID_SVG, path).unwrap();
        
        let info = get_svg_info(path).unwrap();
        
        assert_eq!(info.size_bytes, VALID_SVG.len());
        assert!(info.is_valid);
        assert!(info.line_count > 0);
    }

    #[test]
    fn test_svg_info_summary() {
        let temp_file = NamedTempFile::new().unwrap();
        write_svg(VALID_SVG, temp_file.path()).unwrap();
        
        let info = get_svg_info(temp_file.path()).unwrap();
        let summary = info.summary();
        
        assert!(summary.contains("KB"));
        assert!(summary.contains("Lines"));
        assert!(summary.contains("✓"));
    }

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