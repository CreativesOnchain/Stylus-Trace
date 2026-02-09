use stylus_trace_studio::parser::hostio::{HostIoEvent, HostIoStats, HostIoType, parse_hostio_event};
use stylus_trace_studio::parser::stylus_trace::{extract_total_gas, parse_gas_value, parse_trace};
use serde_json::json;

#[test]
fn test_hostio_event_parsing() {
    let event_json = serde_json::json!({
        "type": "storage_load",
        "gas": 100
    });

    let event = parse_hostio_event(&event_json).unwrap();
    assert_eq!(event.io_type, HostIoType::StorageLoad);
    assert_eq!(event.gas_cost, 100);
}

#[test]
fn test_hostio_type_parsing() {
    assert_eq!(
        "storage_load".parse::<HostIoType>().unwrap(),
        HostIoType::StorageLoad
    );
    assert_eq!(
        "SSTORE".parse::<HostIoType>().unwrap(),
        HostIoType::StorageStore
    );
    assert_eq!("unknown".parse::<HostIoType>().unwrap(), HostIoType::Other);
}

#[test]
fn test_hostio_stats() {
    let mut stats = HostIoStats::new();

    stats.add_event(HostIoEvent {
        io_type: HostIoType::StorageLoad,
        gas_cost: 100,
    });

    stats.add_event(HostIoEvent {
        io_type: HostIoType::StorageLoad,
        gas_cost: 200,
    });

    assert_eq!(stats.count_for_type(HostIoType::StorageLoad), 2);
    assert_eq!(stats.total_gas(), 300);
    assert_eq!(stats.total_calls(), 2);
}

#[test]
fn test_parse_gas_value() {
    assert_eq!(parse_gas_value("1000").unwrap(), 1000);
    assert_eq!(parse_gas_value("0x3e8").unwrap(), 1000);
    assert!(parse_gas_value("invalid").is_err());
}

#[test]
fn test_extract_total_gas() {
    let trace = json!({
        "gasUsed": 50000
    });

    let gas = extract_total_gas(trace.as_object().unwrap()).unwrap();
    assert_eq!(gas, 50000);
}

#[test]
fn test_extract_total_gas_hex() {
    let trace = json!({
        "gasUsed": "0xc350"
    });

    let gas = extract_total_gas(trace.as_object().unwrap()).unwrap();
    assert_eq!(gas, 50000);
}

#[test]
fn test_parse_trace_minimal() {
    let raw_trace = json!({
        "gasUsed": 100000,
        "structLogs": []
    });

    let parsed = parse_trace("0xabc123", &raw_trace).unwrap();
    assert_eq!(parsed.total_gas_used, 1_000_000_000);
    assert_eq!(parsed.transaction_hash, "0xabc123");
}

#[test]
fn test_parse_camelcase_gas_cost() {
    let raw_trace = json!({
        "gasUsed": 100,
        "structLogs": [
            {
                "pc": 0,
                "op": "PUSH1",
                "gas": 1000,
                "gasCost": 3,
                "depth": 1
            }
        ]
    });

    let parsed = parse_trace("0xtest", &raw_trace).unwrap();
    assert_eq!(parsed.execution_steps.len(), 1);
    assert_eq!(parsed.execution_steps[0].gas_cost, 30_000);
}
