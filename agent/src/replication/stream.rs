use crate::models::ChangeEvent;
use crate::replication::decode::Decoder;
use anyhow::{Context, Result};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tokio::sync::{mpsc, RwLock};
use tokio_postgres::{Client, NoTls};
use tokio_stream::Stream;

pub struct ReplicationStream {
    messages: mpsc::Receiver<Result<ChangeEvent>>,
    _replication_handle: tokio::task::JoinHandle<()>,
    client: Client,
    slot_name: String,
}

impl ReplicationStream {
    pub async fn new(
        dsn: &str,
        slot_name: &str,
        _plugin: &str,
        start_lsn: Option<String>,
        decoder: Arc<dyn Decoder + Send + Sync>,
        last_lsn: Arc<RwLock<Option<String>>>,
    ) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls)
            .await
            .context("failed to connect for replication")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("replication connection error: {}", e);
            }
        });

        let start_lsn_str = start_lsn.as_deref().unwrap_or("0/0");

        let query = format!(
            "START_REPLICATION SLOT {} LOGICAL {}",
            slot_name, start_lsn_str
        );

        client
            .simple_query(&query)
            .await
            .context("failed to start replication")?;

        let (tx, rx) = mpsc::channel(1000);
        let _decoder_clone = decoder.clone();
        let _last_lsn_clone = last_lsn.clone();
        let dsn_clone = dsn.to_string();
        let slot_name_clone = slot_name.to_string();
        let start_lsn_clone = start_lsn_str.to_string();

        let replication_handle = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();

            let _sync_client = match postgres::Client::connect(&dsn_clone, postgres::NoTls) {
                Ok(c) => c,
                Err(e) => {
                    rt.block_on(async {
                        let _ = tx
                            .send(Err(anyhow::anyhow!("sync connection error: {}", e)))
                            .await;
                    });
                    return;
                }
            };

            let _query_str = format!(
                "START_REPLICATION SLOT {} LOGICAL {}",
                slot_name_clone, start_lsn_clone
            );

            // Note: postgres crate doesn't support logical replication directly
            // This is a placeholder - proper implementation requires replication protocol library
            // For MVP, this will need to be implemented using raw protocol or a specialized library
            rt.block_on(async {
                let _ = tx
                    .send(Err(anyhow::anyhow!(
                        "Logical replication streaming requires replication protocol implementation"
                    )))
                    .await;
            });
        });

        Ok(Self {
            messages: rx,
            _replication_handle: replication_handle,
            client,
            slot_name: slot_name.to_string(),
        })
    }

    pub async fn send_feedback(&self, lsn: &str) -> Result<()> {
        let query = format!(
            "SELECT pg_logical_replication_slot_advance('{}', '{}')",
            self.slot_name, lsn
        );

        self.client.execute(&query, &[]).await?;
        Ok(())
    }
}

impl Stream for ReplicationStream {
    type Item = Result<ChangeEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        self.messages.poll_recv(cx)
    }
}
