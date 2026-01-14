use crate::models::ChangeEvent;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Decoder: Send + Sync {
    #[allow(dead_code)]
    async fn decode(&self, data: &[u8]) -> Result<Option<ChangeEvent>>;
}
