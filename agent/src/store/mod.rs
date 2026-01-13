pub mod writer;
pub mod checkpoints;
pub mod nodes;
pub mod lsn_time_map;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{Client, NoTls};

#[derive(Clone)]
pub struct StoreManager {
    dsn: String,
    client: Arc<RwLock<Option<Client>>>,
}

impl StoreManager {
    pub async fn new(dsn: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls)
            .await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("store connection error: {}", e);
            }
        });
        
        Ok(Self {
            dsn: dsn.to_string(),
            client: Arc::new(RwLock::new(Some(client))),
        })
    }
    
    pub async fn client(&self) -> Result<Client> {
        let client_opt = self.client.read().await.clone();
        if let Some(client) = client_opt {
            Ok(client)
        } else {
            let (client, connection) = tokio_postgres::connect(&self.dsn, NoTls)
                .await?;
            
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("store connection error: {}", e);
                }
            });
            
            *self.client.write().await = Some(client.clone());
            Ok(client)
        }
    }
    
    pub async fn upsert_node(&self, node_id: &str, agent_version: &str) -> Result<()> {
        let client = self.client().await?;
        client.execute(
            "INSERT INTO pgtimewarp.nodes (node_id, last_seen, agent_version) 
             VALUES ($1, now(), $2)
             ON CONFLICT (node_id) 
             DO UPDATE SET last_seen = now(), agent_version = $2",
            &[&node_id, &agent_version],
        ).await?;
        Ok(())
    }
    
    pub async fn write_lsn_time_map(&self, node_id: &str, lsn: &str) -> Result<()> {
        let client = self.client().await?;
        client.execute(
            "INSERT INTO pgtimewarp.lsn_time_map (node_id, ts, lsn) 
             VALUES ($1, now(), $2::pg_lsn)
             ON CONFLICT (node_id, ts) 
             DO UPDATE SET lsn = $2::pg_lsn",
            &[&node_id, &lsn],
        ).await?;
        Ok(())
    }
}
