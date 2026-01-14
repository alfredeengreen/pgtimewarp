use crate::hashing::compute_pk_hash;
use crate::models::{ChangeEvent, Operation, RowVersion};
use crate::store::writer::Writer;
use crate::store::StoreManager;
use crate::time::now;
use crate::tracking::TrackingManager;
use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;

pub struct Transformer {
    store: Arc<StoreManager>,
    tracking: Arc<TrackingManager>,
    writer: Option<Writer>,
}

impl Transformer {
    pub fn new(store: Arc<StoreManager>, tracking: Arc<TrackingManager>) -> Self {
        Self {
            store,
            tracking,
            writer: None,
        }
    }

    pub async fn transform(&mut self, change: ChangeEvent) -> Result<()> {
        if !self
            .tracking
            .is_tracked(&change.schema, &change.table)
            .await
        {
            return Ok(());
        }

        if self.writer.is_none() {
            let client = self.store.client().await?;
            self.writer = Some(Writer::new(client, 500));
        }

        let relid = match self.tracking.get_relid(&change.schema, &change.table).await {
            Some(id) => id,
            None => return Ok(()),
        };

        let pk_cols = match self
            .tracking
            .get_pk_cols(&change.schema, &change.table)
            .await
        {
            Some(cols) => cols,
            None => return Ok(()),
        };

        let row_data = match change.operation {
            Operation::Insert | Operation::Update => change.after,
            Operation::Delete => change.before,
        };

        let pk_hash = if let Some(ref data) = row_data {
            compute_pk_hash(&pk_cols, data)
        } else {
            return Ok(());
        };

        let timestamp = change.timestamp.unwrap_or_else(now);
        let lsn = change.lsn.clone();

        let version = RowVersion {
            node_id: "default".to_string(),
            relid,
            pk_hash,
            valid_from_ts: timestamp,
            valid_to_ts: None,
            valid_from_lsn: lsn.clone(),
            valid_to_lsn: None,
            op: change.operation,
            row_data,
            txid: change.txid,
            confidence: 2,
        };

        if let Some(ref mut writer) = self.writer {
            writer.add(version).await?;
        }

        Ok(())
    }
}
