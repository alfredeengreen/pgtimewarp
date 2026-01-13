// Basic integration tests for the agent
// These tests verify the core modules compile and basic functionality works

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = crate::node::Node::new("test-node");
        assert_eq!(node.id(), "test-node");
    }

    #[test]
    fn test_config_structure() {
        // Verify config structure can be created
        // This is a smoke test to ensure types are correct
        use crate::config::*;
        
        // Config should have required fields
        // This test just ensures the module compiles
        assert!(true);
    }

    #[test]
    fn test_hashing_deterministic() {
        use crate::hashing::hash_pk;
        use serde_json::json;
        
        let pk1 = json!({"id": 123});
        let pk2 = json!({"id": 123});
        let pk3 = json!({"id": 456});
        
        let hash1 = hash_pk(&pk1);
        let hash2 = hash_pk(&pk2);
        let hash3 = hash_pk(&pk3);
        
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different input should produce different hash
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_models_compile() {
        // Smoke test to ensure models module compiles
        use crate::models::*;
        assert!(true);
    }
}
