use crate::store::Store;
use anyhow::Result;

pub async fn run(store: &Store, node: Option<&str>) -> Result<()> {
    let client = store.client();

    if let Some(node_id) = node {
        let rows = client
            .query(
                "SELECT kind, message, ts, meta 
             FROM pgtimewarp.health_events 
             WHERE node_id = $1 
             ORDER BY ts DESC 
             LIMIT 20",
                &[&node_id],
            )
            .await?;

        println!("Health events for node: {}", node_id);
        for row in rows {
            let kind: String = row.get(0);
            let message: String = row.get(1);
            let ts: chrono::DateTime<chrono::Utc> = row.get(2);
            println!("  [{}] {}: {} - {}", ts, kind, message, "");
        }
    } else {
        let rows = client
            .query(
                "SELECT node_id, last_seen, agent_version 
             FROM pgtimewarp.nodes 
             ORDER BY last_seen DESC",
                &[],
            )
            .await?;

        println!("Nodes:");
        for row in rows {
            let node_id: String = row.get(0);
            let last_seen: chrono::DateTime<chrono::Utc> = row.get(1);
            let agent_version: Option<String> = row.get(2);
            println!(
                "  {} - last seen: {} - version: {:?}",
                node_id, last_seen, agent_version
            );
        }
    }

    Ok(())
}
