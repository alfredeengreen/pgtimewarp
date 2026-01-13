use xxhash_rust::xxh3::Xxh3;

pub fn hash_pk(pk_bytes: &[u8]) -> i64 {
    let mut hasher = Xxh3::new();
    hasher.update(pk_bytes);
    hasher.digest() as i64
}

pub fn encode_pk_canonical(pk_cols: &[String], pk_values: &serde_json::Value) -> Vec<u8> {
    let mut buf = Vec::new();
    
    for (i, col) in pk_cols.iter().enumerate() {
        if i > 0 {
            buf.push(0);
        }
        buf.extend_from_slice(col.as_bytes());
        buf.push(1);
        
        if let Some(val) = pk_values.get(col) {
            let val_str = match val {
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        i.to_string()
                    } else if let Some(f) = n.as_f64() {
                        f.to_string()
                    } else {
                        n.to_string()
                    }
                },
                serde_json::Value::String(s) => s.clone(),
                _ => val.to_string(),
            };
            buf.extend_from_slice(val_str.as_bytes());
        }
    }
    
    buf
}

pub fn compute_pk_hash(pk_cols: &[String], pk_values: &serde_json::Value) -> i64 {
    let canonical = encode_pk_canonical(pk_cols, pk_values);
    hash_pk(&canonical)
}
