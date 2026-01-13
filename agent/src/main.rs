use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, error};

mod config;
mod logging;
mod node;
mod replication;
mod tracking;
mod store;
mod pipeline;
mod models;
mod hashing;
mod time;

use config::Config;
use node::Node;
use replication::ReplicationManager;
use tracking::TrackingManager;
use store::{StoreManager, checkpoints};
use pipeline::Pipeline;
use tokio_stream::StreamExt;

#[derive(Parser)]
#[command(name = "pgtimewarp-agent")]
#[command(about = "PostgreSQL time travel WAL consumer agent")]
struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    logging::init()?;
    
    info!("starting pgtimewarp agent");
    
    let config = Config::load(&args.config)?;
    info!("loaded configuration from {:?}", args.config);
    
    let node = Node::new(&config.node_id);
    let store = Arc::new(StoreManager::new(&config.store.dsn).await?);
    let tracking = Arc::new(TrackingManager::new(store.clone(), node.id().to_string()).await?);
    let replication = Arc::new(ReplicationManager::new(&config.source, &config.store.dsn).await?);
    let pipeline = Arc::new(Pipeline::new(
        store.clone(),
        tracking.clone(),
        config.limits.batch_size,
        config.limits.max_queue,
    ));
    
    store.upsert_node(node.id(), &config.agent_version).await?;
    
    let mut shutdown = tokio::signal::ctrl_c();
    
    tokio::select! {
        result = run_agent(node, store, tracking, replication, pipeline, config) => {
            if let Err(e) = result {
                error!("agent error: {}", e);
                return Err(e);
            }
        }
        _ = shutdown => {
            info!("shutdown signal received");
        }
    }
    
    info!("agent stopped");
    Ok(())
}

async fn run_agent(
    node: Node,
    store: Arc<StoreManager>,
    tracking: Arc<TrackingManager>,
    replication: Arc<ReplicationManager>,
    pipeline: Arc<Pipeline>,
    config: Config,
) -> Result<()> {
    let node_id = node.id().to_string();
    
    let mut tracking_refresh = tokio::time::interval(
        std::time::Duration::from_secs(config.intervals.refresh_tracked_s)
    );
    let mut node_heartbeat = tokio::time::interval(
        std::time::Duration::from_secs(30)
    );
    let mut lsn_time_map_interval = tokio::time::interval(
        std::time::Duration::from_secs(5)
    );
    
    replication.ensure_slot().await?;
    
    let mut stream = replication.start_stream().await?;
    let mut last_feedback = tokio::time::Instant::now();
    let mut last_checkpoint = tokio::time::Instant::now();
    let store_client = store.client().await?;
    
    loop {
        tokio::select! {
            _ = tracking_refresh.tick() => {
                if let Err(e) = tracking.refresh().await {
                    error!("tracking refresh error: {}", e);
                }
            }
            _ = node_heartbeat.tick() => {
                if let Err(e) = store.upsert_node(&node_id, &config.agent_version).await {
                    error!("node heartbeat error: {}", e);
                }
            }
            _ = lsn_time_map_interval.tick() => {
                if let Some(last_lsn) = replication.last_lsn().await {
                    if let Err(e) = store.write_lsn_time_map(&node_id, &last_lsn).await {
                        error!("lsn time map write error: {}", e);
                    }
                }
            }
            msg = stream.next() => {
                match msg {
                    Some(Ok(change)) => {
                        if let Err(e) = pipeline.process_change(change).await {
                            error!("pipeline error: {}", e);
                        }
                        
                        if last_feedback.elapsed().as_secs() >= 10 {
                            if let Some(lsn) = replication.last_lsn().await {
                                if let Err(e) = stream.send_feedback(&lsn).await {
                                    error!("feedback error: {}", e);
                                }
                            }
                            last_feedback = tokio::time::Instant::now();
                        }
                        
                        if last_checkpoint.elapsed().as_secs() >= 30 {
                            if let Some(lsn) = replication.last_lsn().await {
                                if let Err(e) = checkpoints::update_checkpoint(
                                    &store_client,
                                    &node_id,
                                    &config.source.slot_name,
                                    &lsn,
                                ).await {
                                    error!("checkpoint update error: {}", e);
                                }
                            }
                            last_checkpoint = tokio::time::Instant::now();
                        }
                    }
                    Some(Err(e)) => {
                        error!("replication stream error: {}", e);
                        break;
                    }
                    None => {
                        info!("replication stream ended");
                        break;
                    }
                }
            }
        }
    }
    
    Ok(())
}

