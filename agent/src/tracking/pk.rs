use crate::hashing::{encode_pk_canonical, hash_pk};
use serde_json::Value;

#[allow(dead_code)]
pub fn compute_pk_hash(pk_cols: &[String], row_data: &Value) -> i64 {
    let pk_values = extract_pk_values(pk_cols, row_data);
    let pk_bytes = encode_pk_canonical(pk_cols, &pk_values);
    hash_pk(&pk_bytes)
}

#[allow(dead_code)]
fn extract_pk_values(pk_cols: &[String], row_data: &Value) -> Value {
    if let Some(obj) = row_data.as_object() {
        let mut pk_obj = serde_json::Map::new();
        for col in pk_cols {
            if let Some(val) = obj.get(col) {
                pk_obj.insert(col.clone(), val.clone());
            }
        }
        Value::Object(pk_obj)
    } else {
        Value::Null
    }
}
