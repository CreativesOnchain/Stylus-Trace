use stylus_trace_studio::commands::{validate_args, CaptureArgs};

#[test]
fn test_validate_args_valid() {
    let args = CaptureArgs {
        rpc_url: "http://localhost:8547".to_string(),
        transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        ..Default::default()
    };

    assert!(validate_args(&args).is_ok());
}

#[test]
fn test_validate_args_empty_rpc() {
    let args = CaptureArgs {
        rpc_url: String::new(),
        transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        ..Default::default()
    };

    assert!(validate_args(&args).is_err());
}

#[test]
fn test_validate_args_invalid_rpc_scheme() {
    let args = CaptureArgs {
        rpc_url: "ftp://localhost:8547".to_string(),
        transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        ..Default::default()
    };

    assert!(validate_args(&args).is_err());
}

#[test]
fn test_validate_args_empty_tx_hash() {
    let args = CaptureArgs {
        rpc_url: "http://localhost:8547".to_string(),
        transaction_hash: String::new(),
        ..Default::default()
    };

    assert!(validate_args(&args).is_err());
}

#[test]
fn test_validate_args_short_tx_hash() {
    let args = CaptureArgs {
        rpc_url: "http://localhost:8547".to_string(),
        transaction_hash: "0x1234".to_string(),
        ..Default::default()
    };

    assert!(validate_args(&args).is_err());
}

#[test]
fn test_validate_args_invalid_hex() {
    let args = CaptureArgs {
        rpc_url: "http://localhost:8547".to_string(),
        transaction_hash: "0xGGGG567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        ..Default::default()
    };

    assert!(validate_args(&args).is_err());
}

#[test]
fn test_validate_args_tx_hash_without_prefix() {
    let args = CaptureArgs {
        rpc_url: "http://localhost:8547".to_string(),
        transaction_hash: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        ..Default::default()
    };

    assert!(validate_args(&args).is_ok());
}

#[test]
fn test_validate_args_top_paths_zero() {
    let args = CaptureArgs {
        rpc_url: "http://localhost:8547".to_string(),
        transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        top_paths: 0,
        ..Default::default()
    };

    assert!(validate_args(&args).is_err());
}

#[test]
fn test_validate_args_top_paths_too_large() {
    let args = CaptureArgs {
        rpc_url: "http://localhost:8547".to_string(),
        transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        top_paths: 2000,
        ..Default::default()
    };

    assert!(validate_args(&args).is_err());
}
