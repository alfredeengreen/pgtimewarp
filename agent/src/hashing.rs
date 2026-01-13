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
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(b) => if *b { "true" } else { "false" },
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        format!("{}", i).as_str()
                    } else if let Some(f) = n.as_f64() {
                        format!("{}", f).as_str()
                    } else {
                        n.to_string().as_str()
                    }
                },
                serde_json::Value::String(s) => s.as_str(),
                _ => val.to_string().as_str(),
            };
            buf.extend_from_slice(val_str.as_bytes());
        }
    }
    
    buf
}
