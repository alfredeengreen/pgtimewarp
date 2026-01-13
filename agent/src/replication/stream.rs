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
    _replication_handle: std::thread::JoinHandle<()>,
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
            slot_name,
            start_lsn_str
        );
        
        client
            .simple_query(&query)
            .await
            .context("failed to start replication")?;
        
        let (tx, rx) = mpsc::channel(1000);
        let decoder_clone = decoder.clone();
        let last_lsn_clone = last_lsn.clone();
        let slot_name_clone = slot_name.to_string();
        let dsn_clone = dsn.to_string();
        let start_lsn_clone = start_lsn_str.to_string();
        
        let rt = tokio::runtime::Handle::current();
        let replication_handle = std::thread::spawn(move || {
            let sync_client = match postgres::Client::connect(&dsn_clone, postgres::NoTls) {
                Ok(c) => c,
                Err(e) => {
                    rt.block_on(async {
                        let _ = tx.send(Err(anyhow::anyhow!("sync connection error: {}", e))).await;
                    });
                    return;
                }
            };
            
            let query_str = format!(
                "START_REPLICATION SLOT {} LOGICAL {}",
                slot_name_clone, start_lsn_clone
            );
            
            let mut replication_stream = match sync_client.copy_both_simple::<&[u8], Vec<u8>>(&query_str) {
                Ok(stream) => stream,
                Err(e) => {
                    rt.block_on(async {
                        let _ = tx.send(Err(anyhow::anyhow!("replication start error: {}", e))).await;
                    });
                    return;
                }
            };
            
            loop {
                match replication_stream.read() {
                    Ok(Some(msg)) => {
                        let decoder = decoder_clone.clone();
                        let msg_clone = msg.clone();
                        let last_lsn = last_lsn_clone.clone();
                        let tx_clone = tx.clone();
                        
                        rt.spawn(async move {
                            match decoder.decode(&msg_clone).await {
                                Ok(Some(mut event)) => {
                                    if let Some(ref lsn) = event.lsn {
                                        if !lsn.is_empty() {
                                            *last_lsn.write().await = Some(lsn.clone());
                                        }
                                    }
                                    if tx_clone.send(Ok(event)).await.is_err() {
                                        return;
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    if tx_clone.send(Err(anyhow::anyhow!("decode error: {}", e))).await.is_err() {
                                        return;
                                    }
                                }
                            }
                        });
                    }
                    Ok(None) => break,
                    Err(e) => {
                        rt.block_on(async {
                            let _ = tx.send(Err(anyhow::anyhow!("replication read error: {}", e))).await;
                        });
                        break;
                    }
                }
            }
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
    
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.messages.poll_recv(cx)
    }
}
