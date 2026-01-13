use crate::models::ChangeEvent;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Decoder: Send + Sync {
    async fn decode(&self, data: &[u8]) -> Result<Option<ChangeEvent>>;
}
