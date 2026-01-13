use anyhow::Result;
use tokio_postgres::Client;

pub async fn upsert_node(
    client: &Client,
    node_id: &str,
    agent_version: &str,
) -> Result<()> {
    client.execute(
        "INSERT INTO pgtimewarp.nodes (node_id, last_seen, agent_version) 
         VALUES ($1, now(), $2)
         ON CONFLICT (node_id) 
         DO UPDATE SET last_seen = now(), agent_version = $2",
        &[&node_id, &agent_version],
    ).await?;
    Ok(())
}
