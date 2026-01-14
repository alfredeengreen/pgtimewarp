use crate::config::{SourceConfig, Wal2JsonOptions};
use crate::models::{ChangeEvent, Operation};
use crate::replication::decode::Decoder;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;

pub struct Wal2JsonDecoder {
    options: Wal2JsonOptions,
}

impl Wal2JsonDecoder {
    pub fn new(config: &SourceConfig) -> Result<Self> {
        let options = config
            .wal2json_options
            .clone()
            .unwrap_or_else(|| SourceConfig::default_wal2json_options());

        Ok(Self { options })
    }
}

#[async_trait]
impl Decoder for Wal2JsonDecoder {
    async fn decode(&self, data: &[u8]) -> Result<Option<ChangeEvent>> {
        let json: Value =
            serde_json::from_slice(data).context("failed to parse wal2json message")?;

        let change = if let Some(change_array) = json.get("change").and_then(|v| v.as_array()) {
            if change_array.is_empty() {
                return Ok(None);
            }
            &change_array[0]
        } else if json.get("change").is_some() {
            json.get("change").unwrap()
        } else {
            return Ok(None);
        };

        let schema = change
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or("public")
            .to_string();

        let table = change
            .get("table")
            .and_then(|v| v.as_str())
            .context("missing table name")?
            .to_string();

        let kind = change
            .get("kind")
            .and_then(|v| v.as_str())
            .context("missing operation kind")?;

        let operation = match kind {
            "insert" => Operation::Insert,
            "update" => Operation::Update,
            "delete" => Operation::Delete,
            _ => anyhow::bail!("unknown operation kind: {}", kind),
        };

        let lsn = if self.options.include_lsn {
            json.get("lsn")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let timestamp = if self.options.include_timestamp {
            json.get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
        } else {
            None
        };

        let txid = json.get("xid").and_then(|v| v.as_i64());

        let before = change
            .get("oldkeys")
            .and_then(|v| v.get("oldkeys"))
            .cloned();

        let after = change.get("columnnames").and_then(|names| {
            change.get("columnvalues").and_then(|values| {
                if let (Some(names_arr), Some(values_arr)) = (names.as_array(), values.as_array()) {
                    if names_arr.len() == values_arr.len() {
                        let mut obj = serde_json::Map::new();
                        for (name, value) in names_arr.iter().zip(values_arr.iter()) {
                            if let Some(name_str) = name.as_str() {
                                obj.insert(name_str.to_string(), value.clone());
                            }
                        }
                        Some(Value::Object(obj))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        });

        Ok(Some(ChangeEvent {
            schema,
            table,
            operation,
            lsn,
            timestamp,
            before,
            after,
            txid,
        }))
    }
}
