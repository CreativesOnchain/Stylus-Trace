//! JSON profile output writer.
//!
//! Writes Profile structs to JSON files with proper formatting.

use crate::parser::schema::Profile;
use crate::utils::error::OutputError;
use log::{debug, info};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Write a profile to a JSON file
///
/// **Public** - main entry point for JSON output
///
/// # Arguments
/// * `profile` - Profile data to write
/// * `output_path` - Path to output JSON file
///
/// # Returns
/// Ok if file written successfully
///
/// # Errors
/// * `OutputError::WriteFailed` - I/O error during write
/// * `OutputError::SerializationFailed` - JSON serialization error
/// * `OutputError::InvalidPath` - Path cannot be created or is invalid
///
/// # Example
/// ```ignore
/// let profile = to_profile(&parsed_trace, hot_paths);
/// write_profile(&profile, "profile.json")?;
/// ```
pub fn write_profile(profile: &Profile, output_path: impl AsRef<Path>) -> Result<(), OutputError> {
    let output_path = output_path.as_ref();
    
    info!("Writing profile to: {}", output_path.display());
    
    // Validate path
    validate_output_path(output_path)?;
    
    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            debug!("Creating parent directories: {}", parent.display());
            std::fs::create_dir_all(parent)
                .map_err(|e| OutputError::InvalidPath(format!(
                    "Cannot create directory {}: {}",
                    parent.display(),
                    e
                )))?;
        }
    }
    
    // Open file for writing
    let file = File::create(output_path)
        .map_err(OutputError::WriteFailed)?;
    
    let writer = BufWriter::new(file);
    
    // Serialize to JSON with pretty printing
    serde_json::to_writer_pretty(writer, profile)
        .map_err(OutputError::SerializationFailed)?;
    
    info!("Profile written successfully ({} bytes)", 
          calculate_file_size(output_path));
    
    Ok(())
}

// /// Write profile as compact JSON (no formatting)
// ///
// /// **Public** - useful for when file size matters (CI artifacts, etc.)
// ///
// /// # Arguments
// /// * `profile` - Profile data to write
// /// * `output_path` - Path to output JSON file
// ///
// /// # Returns
// /// Ok if file written successfully
/*
pub fn write_profile_compact(
    profile: &Profile,
    output_path: impl AsRef<Path>,
) -> Result<(), OutputError> {
    // ...
    Ok(())
}
*/

// /// Write profile to a string (for testing or in-memory use)
// ///
// /// **Public** - useful for tests and debugging
// ///
// /// # Arguments
// /// * `profile` - Profile to serialize
// ///
// /// # Returns
// // /// JSON string
/*
pub fn profile_to_string(profile: &Profile) -> Result<String, OutputError> {
    serde_json::to_string_pretty(profile)
        .map_err(OutputError::SerializationFailed)
}
*/

/// Validate that output path is writable
///
/// **Private** - internal validation
fn validate_output_path(path: &Path) -> Result<(), OutputError> {
    // Check if path is absolute or relative
    if path.as_os_str().is_empty() {
        return Err(OutputError::InvalidPath("Path is empty".to_string()));
    }
    
    // Check if we're trying to overwrite a directory
    if path.exists() && path.is_dir() {
        return Err(OutputError::InvalidPath(format!(
            "Path is a directory: {}",
            path.display()
        )));
    }
    
    // Check if parent directory exists or can be created
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            // This is okay - we'll create it
            debug!("Parent directory will be created: {}", parent.display());
        }
    }
    
    Ok(())
}

/// Calculate file size in bytes
///
/// **Private** - internal utility
fn calculate_file_size(path: &Path) -> u64 {
    std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0)
}

/// Read a profile from a JSON file
///
/// **Public** - useful for validation, diff, and testing
///
/// # Arguments
/// * `input_path` - Path to JSON file
///
/// # Returns
/// Parsed Profile
///
/// # Errors
/// * `OutputError::WriteFailed` - File read error (reusing WriteFailed for I/O)
/// * `OutputError::SerializationFailed` - JSON parse error
pub fn read_profile(input_path: impl AsRef<Path>) -> Result<Profile, OutputError> {
    let input_path = input_path.as_ref();
    
    debug!("Reading profile from: {}", input_path.display());
    
    let file = File::open(input_path)
        .map_err(OutputError::WriteFailed)?;
    
    let profile: Profile = serde_json::from_reader(file)
        .map_err(OutputError::SerializationFailed)?;
    
    debug!("Profile loaded: version {}, tx {}", 
           profile.version, 
           profile.transaction_hash);
    
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::schema::{Profile, HostIoSummary, HotPath};
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    fn create_test_profile() -> Profile {
        Profile {
            version: "1.0.0".to_string(),
            transaction_hash: "0xtest123".to_string(),
            total_gas: 100000,
            hostio_summary: HostIoSummary {
                total_calls: 10,
                by_type: HashMap::new(),
                total_hostio_gas: 5000,
            },
            hot_paths: vec![
                HotPath {
                    stack: "main;execute".to_string(),
                    gas: 50000,
                    percentage: 50.0,
                    source_hint: None,
                }
            ],
            generated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_write_and_read_profile() {
        let profile = create_test_profile();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Write
        write_profile(&profile, path).unwrap();
        
        // Read back
        let loaded = read_profile(path).unwrap();
        
        assert_eq!(loaded.version, profile.version);
        assert_eq!(loaded.transaction_hash, profile.transaction_hash);
        assert_eq!(loaded.total_gas, profile.total_gas);
    }

/*
    #[test]
    fn test_write_profile_compact() {
        // ...
    }

    #[test]
    fn test_profile_to_string() {
        // ...
    }
*/

    #[test]
    fn test_validate_output_path_empty() {
        let result = validate_output_path(Path::new(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_output_path_directory() {
        // Try to write to a directory path
        let temp_dir = tempfile::tempdir().unwrap();
        let result = validate_output_path(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_write_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested/dirs/profile.json");
        
        let profile = create_test_profile();
        write_profile(&profile, &nested_path).unwrap();
        
        assert!(nested_path.exists());
    }
}