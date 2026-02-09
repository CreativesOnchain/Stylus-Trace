use stylus_trace_studio::flamegraph::generator::{NodeCategory, get_truncated_name};

#[test]
fn test_node_category() {
    assert_eq!(NodeCategory::from_name("root"), NodeCategory::Root);
    assert_eq!(
        NodeCategory::from_name("storage_load"),
        NodeCategory::StorageNormal
    );
    assert_eq!(
        NodeCategory::from_name("SLOAD"),
        NodeCategory::StorageNormal
    );
    assert_eq!(
        NodeCategory::from_name("storage_flush"),
        NodeCategory::StorageExpensive
    );
    assert_eq!(
        NodeCategory::from_name("SSTORE"),
        NodeCategory::StorageExpensive
    );
    assert_eq!(NodeCategory::from_name("keccak256"), NodeCategory::Crypto);
    assert_eq!(NodeCategory::from_name("memory_read"), NodeCategory::Memory);
    assert_eq!(NodeCategory::from_name("call"), NodeCategory::Call);
    assert_eq!(
        NodeCategory::from_name("StylusRuntime"),
        NodeCategory::System
    );
    assert_eq!(NodeCategory::from_name("random_fn"), NodeCategory::UserCode);
}

#[test]
fn test_get_truncated_name() {
    // Not enough width
    assert_eq!(get_truncated_name("long_function_name", 30.0), None);

    // Exact fit or enough room
    assert_eq!(get_truncated_name("abc", 40.0), Some("abc".to_string()));

    // Truncation needed
    let name = "very_long_function_name";
    let truncated = get_truncated_name(name, 50.0).unwrap();
    assert!(truncated.ends_with("..."));
    assert!(truncated.len() < name.len());
}
