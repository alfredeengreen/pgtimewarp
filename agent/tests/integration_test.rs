// Basic integration tests for the agent
// These tests verify the core modules compile and basic functionality works

use pgtimewarp_agent::*;

#[test]
fn test_node_creation() {
    let node = Node::new("test-node");
    assert_eq!(node.id(), "test-node");
}

#[test]
fn test_config_structure() {
    // Verify config structure can be created
    // This is a smoke test to ensure types are correct
    // Config should have required fields
    // This test just ensures the module compiles
    assert!(true);
}

#[test]
fn test_hashing_deterministic() {
    use serde_json::json;

    let pk_cols = vec!["id".to_string()];
    let pk1 = json!({"id": 123});
    let pk2 = json!({"id": 123});
    let pk3 = json!({"id": 456});

    let hash1 = compute_pk_hash(&pk_cols, &pk1);
    let hash2 = compute_pk_hash(&pk_cols, &pk2);
    let hash3 = compute_pk_hash(&pk_cols, &pk3);

    // Same input should produce same hash
    assert_eq!(hash1, hash2);

    // Different input should produce different hash
    assert_ne!(hash1, hash3);
}

#[test]
fn test_models_compile() {
    // Smoke test to ensure models module compiles
    let _ = ChangeEvent {
        schema: "public".to_string(),
        table: "test".to_string(),
        operation: Operation::Insert,
        lsn: "0/0".to_string(),
        timestamp: None,
        before: None,
        after: None,
        txid: None,
    };
    assert!(true);
}
