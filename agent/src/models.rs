use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub schema: String,
    pub table: String,
    pub operation: Operation,
    pub lsn: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
    pub txid: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

impl Operation {
    pub fn as_i16(self) -> i16 {
        match self {
            Operation::Insert => 0,
            Operation::Update => 1,
            Operation::Delete => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RowVersion {
    pub node_id: String,
    pub relid: u32,
    pub pk_hash: i64,
    pub valid_from_ts: DateTime<Utc>,
    pub valid_to_ts: Option<DateTime<Utc>>,
    pub valid_from_lsn: String,
    pub valid_to_lsn: Option<String>,
    pub op: Operation,
    pub row_data: Option<serde_json::Value>,
    pub txid: Option<i64>,
    pub confidence: i16,
}
