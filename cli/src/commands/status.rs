use crate::store::Store;
use anyhow::Result;

pub async fn run(store: &Store, node: Option<&str>) -> Result<()> {
    let client = store.client();

    if let Some(node_id) = node {
        let rows = client
            .query(
                "SELECT schema_name, table_name, status, retention_hours, created_at 
             FROM pgtimewarp.tracked_relations 
             WHERE node_id = $1 
             ORDER BY schema_name, table_name",
                &[&node_id],
            )
            .await?;

        println!("Tracked tables for node: {}", node_id);
        for row in rows {
            let schema: String = row.get(0);
            let table: String = row.get(1);
            let status: i16 = row.get(2);
            let retention: i32 = row.get(3);
            let created: chrono::DateTime<chrono::Utc> = row.get(4);
            let status_str = match status {
                0 => "active",
                1 => "paused",
                2 => "needs_reinit",
                _ => "unknown",
            };
            println!(
                "  {}.{} - {} - retention: {}h - created: {}",
                schema, table, status_str, retention, created
            );
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

        let rows = client
            .query(
                "SELECT node_id, schema_name, table_name, status 
             FROM pgtimewarp.tracked_relations 
             ORDER BY node_id, schema_name, table_name",
                &[],
            )
            .await?;

        println!("\nTracked tables:");
        for row in rows {
            let node_id: String = row.get(0);
            let schema: String = row.get(1);
            let table: String = row.get(2);
            let status: i16 = row.get(3);
            let status_str = match status {
                0 => "active",
                1 => "paused",
                2 => "needs_reinit",
                _ => "unknown",
            };
            println!("  {}: {}.{} - {}", node_id, schema, table, status_str);
        }
    }

    Ok(())
}
