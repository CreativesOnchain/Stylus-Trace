use stylus_trace_studio::rpc::client::normalize_tx_hash;

#[test]
fn test_normalize_tx_hash() {
    assert_eq!(normalize_tx_hash("abc123"), "0xabc123");
    assert_eq!(normalize_tx_hash("0xdef456"), "0xdef456");
}
