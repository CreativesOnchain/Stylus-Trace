use stylus_trace_studio::aggregator::stack_builder::{CollapsedStack, map_hostio_to_label};
use stylus_trace_studio::aggregator::metrics::{calculate_gas_distribution, calculate_hot_paths, create_hot_path};
use stylus_trace_studio::parser::HostIoType;

#[test]
fn test_map_hostio_to_label() {
    assert_eq!(
        map_hostio_to_label(HostIoType::StorageLoad),
        "storage_load_bytes32"
    );
    assert_eq!(map_hostio_to_label(HostIoType::Call), "call");
}

#[test]
fn test_calculate_hot_paths() {
    let stacks = vec![
        CollapsedStack::new("main;execute".to_string(), 5000, Some(0x100)),
        CollapsedStack::new("main;storage".to_string(), 3000, Some(0x200)),
        CollapsedStack::new("main;compute".to_string(), 2000, Some(0x300)),
    ];

    let hot_paths = calculate_hot_paths(&stacks, 10000, 2);

    assert_eq!(hot_paths.len(), 2);
    assert_eq!(hot_paths[0].stack, "main;execute");
    assert_eq!(hot_paths[0].gas, 5000);
    assert_eq!(hot_paths[0].percentage, 50.0);
}

#[test]
fn test_calculate_gas_distribution() {
    let stacks = vec![
        CollapsedStack::new("stack1".to_string(), 8500, Some(0x1)),
        CollapsedStack::new("stack2".to_string(), 1000, Some(0x2)),
        CollapsedStack::new("stack3".to_string(), 250, Some(0x3)),
        CollapsedStack::new("stack4".to_string(), 250, Some(0x4)),
    ];

    let dist = calculate_gas_distribution(&stacks);

    assert_eq!(dist.total_gas, 10000);
    assert_eq!(dist.stack_count, 4);
    assert_eq!(dist.mean_gas_per_stack, 2500);
}

#[test]
fn test_gas_distribution_empty() {
    let stacks: Vec<CollapsedStack> = vec![];
    let dist = calculate_gas_distribution(&stacks);
    assert_eq!(dist.total_gas, 0);
    assert_eq!(dist.stack_count, 0);
}

#[test]
fn test_create_hot_path() {
    let stack = CollapsedStack::new("test;path".to_string(), 2500, Some(0x42));
    let hot_path = create_hot_path(&stack, 10000);

    assert_eq!(hot_path.stack, "test;path");
    assert_eq!(hot_path.gas, 2500);
    assert_eq!(hot_path.percentage, 25.0);
    assert!(hot_path.source_hint.is_some());
    assert_eq!(
        hot_path.source_hint.unwrap().function,
        Some("0x42".to_string())
    );
}
