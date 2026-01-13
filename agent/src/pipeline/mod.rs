pub mod transform;
pub mod backpressure;

use crate::models::ChangeEvent;
use crate::store::StoreManager;
use crate::tracking::TrackingManager;
use anyhow::Result;
use std::sync::Arc;

pub struct Pipeline {
    store: Arc<StoreManager>,
    tracking: Arc<TrackingManager>,
    transform: Arc<tokio::sync::Mutex<transform::Transformer>>,
}

impl Pipeline {
    pub fn new(
        store: Arc<StoreManager>,
        tracking: Arc<TrackingManager>,
        _batch_size: usize,
        _max_queue: usize,
    ) -> Self {
        let transform = Arc::new(tokio::sync::Mutex::new(
            transform::Transformer::new(store.clone(), tracking.clone())
        ));
        
        Self {
            store,
            tracking,
            transform,
        }
    }
    
    pub async fn process_change(&self, change: ChangeEvent) -> Result<()> {
        let mut transformer = self.transform.lock().await;
        transformer.transform(change).await
    }
}
