pub mod slot;
pub mod stream;
pub mod decode;
pub mod wal2json;
pub mod pgoutput;

pub use slot::SlotManager;
pub use stream::ReplicationStream;
pub use decode::Decoder;

use crate::config::SourceConfig;
use crate::models::ChangeEvent;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ReplicationManager {
    source_config: Arc<SourceConfig>,
    store_dsn: String,
    slot_manager: Arc<SlotManager>,
    decoder: Arc<dyn Decoder + Send + Sync>,
    last_lsn: Arc<RwLock<Option<String>>>,
}

impl ReplicationManager {
    pub async fn new(source_config: &SourceConfig, store_dsn: &str) -> Result<Self> {
        let slot_manager = Arc::new(SlotManager::new(source_config)?);
        let decoder: Arc<dyn Decoder + Send + Sync> = match source_config.plugin.as_str() {
            "wal2json" => Arc::new(wal2json::Wal2JsonDecoder::new(source_config)?),
            "pgoutput" => Arc::new(pgoutput::PgOutputDecoder::new(source_config)?),
            _ => anyhow::bail!("unsupported replication plugin: {}", source_config.plugin),
        };
        
        Ok(Self {
            source_config: Arc::new(source_config.clone()),
            store_dsn: store_dsn.to_string(),
            slot_manager,
            decoder,
            last_lsn: Arc::new(RwLock::new(None)),
        })
    }
    
    pub async fn ensure_slot(&self) -> Result<()> {
        self.slot_manager.ensure_slot().await
    }
    
    pub async fn start_stream(&self) -> Result<ReplicationStream> {
        let last_lsn = self.slot_manager.get_last_lsn(&self.store_dsn).await?;
        let stream = ReplicationStream::new(
            &self.source_config.dsn,
            &self.source_config.slot_name,
            &self.source_config.plugin,
            last_lsn,
            self.decoder.clone(),
            self.last_lsn.clone(),
        ).await?;
        Ok(stream)
    }
    
    pub async fn last_lsn(&self) -> Option<String> {
        self.last_lsn.read().await.clone()
    }
}
