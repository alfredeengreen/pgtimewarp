use crate::models::{Operation, RowVersion};
use anyhow::Result;
use tokio_postgres::Client;

pub struct Writer {
    client: Client,
    batch: Vec<RowVersion>,
    batch_size: usize,
}

impl Writer {
    pub fn new(client: Client, batch_size: usize) -> Self {
        Self {
            client,
            batch: Vec::with_capacity(batch_size),
            batch_size,
        }
    }

    pub async fn add(&mut self, version: RowVersion) -> Result<()> {
        self.batch.push(version);

        if self.batch.len() >= self.batch_size {
            self.flush().await?;
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }

        let tx = self.client.transaction().await?;

        for version in &self.batch {
            if version.op != Operation::Insert {
                tx.execute(
                    "UPDATE pgtimewarp.row_versions 
                     SET valid_to_lsn = $1::pg_lsn, valid_to_ts = $2 
                     WHERE node_id = $3 
                       AND relid = $4 
                       AND pk_hash = $5 
                       AND valid_to_lsn IS NULL 
                       AND valid_from_lsn < $1::pg_lsn",
                    &[
                        &version.valid_from_lsn,
                        &version.valid_from_ts,
                        &version.node_id,
                        &(version.relid as i32),
                        &version.pk_hash,
                    ],
                )
                .await?;
            }

            tx.execute(
                "INSERT INTO pgtimewarp.row_versions 
                 (node_id, relid, pk_hash, valid_from_ts, valid_to_ts, 
                  valid_from_lsn, valid_to_lsn, op, row_data, txid, confidence) 
                 VALUES ($1, $2, $3, $4, $5, $6::pg_lsn, $7::pg_lsn, $8, $9, $10, $11)",
                &[
                    &version.node_id,
                    &(version.relid as i32),
                    &version.pk_hash,
                    &version.valid_from_ts,
                    &version.valid_to_ts,
                    &version.valid_from_lsn,
                    &version.valid_to_lsn.as_ref().map(|s| s.as_str()),
                    &version.op.as_i16(),
                    &version.row_data,
                    &version.txid,
                    &version.confidence,
                ],
            )
            .await?;
        }

        tx.commit().await?;
        self.batch.clear();

        Ok(())
    }
}
