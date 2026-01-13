use crate::config::SourceConfig;
use crate::models::ChangeEvent;
use crate::replication::decode::Decoder;
use anyhow::Result;
use async_trait::async_trait;

pub struct PgOutputDecoder {
    _config: SourceConfig,
}

impl PgOutputDecoder {
    pub fn new(config: &SourceConfig) -> Result<Self> {
        Ok(Self {
            _config: config.clone(),
        })
    }
}

#[async_trait]
impl Decoder for PgOutputDecoder {
    async fn decode(&self, _data: &[u8]) -> Result<Option<ChangeEvent>> {
        anyhow::bail!("pgoutput decoder not yet implemented")
    }
}
