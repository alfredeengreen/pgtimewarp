pub mod catalog;
pub mod schema;
pub mod pk;

use crate::store::StoreManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TrackingManager {
    store: Arc<StoreManager>,
    node_id: String,
    catalog: Arc<RwLock<catalog::Catalog>>,
}

impl TrackingManager {
    pub async fn new(store: Arc<StoreManager>, node_id: String) -> Result<Self> {
        let catalog = Arc::new(RwLock::new(catalog::Catalog::new(store.clone(), node_id.clone()).await?));
        
        Ok(Self {
            store,
            node_id,
            catalog,
        })
    }
    
    pub async fn refresh(&self) -> Result<()> {
        self.catalog.write().await.refresh().await
    }
    
    pub async fn is_tracked(&self, schema: &str, table: &str) -> bool {
        self.catalog.read().await.is_tracked(schema, table)
    }
    
    pub async fn get_relid(&self, schema: &str, table: &str) -> Option<u32> {
        self.catalog.read().await.get_relid(schema, table)
    }
    
    pub async fn get_pk_cols(&self, schema: &str, table: &str) -> Option<Vec<String>> {
        self.catalog.read().await.get_pk_cols(schema, table)
    }
}
