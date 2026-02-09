use std::collections::HashMap;
use std::path::Path;
use stylus_trace_studio::output::validate_path;
use stylus_trace_studio::output::{read_profile, write_profile, write_svg};
use stylus_trace_studio::parser::schema::{HostIoSummary, HotPath, Profile};
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
        hot_paths: vec![HotPath {
            stack: "main;execute".to_string(),
            gas: 50000,
            percentage: 50.0,
            source_hint: None,
        }],
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

#[test]
fn test_validate_output_path_empty() {
    let result = validate_path(Path::new(""));
    assert!(result.is_err());
}

#[test]
fn test_validate_output_path_directory() {
    // Try to write to a directory path
    let temp_dir = tempfile::tempdir().unwrap();
    let result = validate_path(temp_dir.path());
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

#[test]
fn test_write_and_read_svg() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let valid_svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect x="0" y="0" width="100" height="100" fill="red"/>
</svg>"#;

    write_svg(valid_svg, path).unwrap();
}

#[test]
fn test_svg_write_creates_parent_dirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let nested_path = temp_dir.path().join("nested/dirs/flamegraph.svg");
    let valid_svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect x="0" y="0" width="100" height="100" fill="red"/>
</svg>"#;

    write_svg(valid_svg, &nested_path).unwrap();

    assert!(nested_path.exists());
}
