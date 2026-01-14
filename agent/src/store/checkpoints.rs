use anyhow::Result;
use tokio_postgres::Client;

pub async fn update_checkpoint(
    client: &Client,
    node_id: &str,
    slot_name: &str,
    lsn: &str,
) -> Result<()> {
    client
        .execute(
            "INSERT INTO pgtimewarp.wal_checkpoints (node_id, slot_name, last_lsn, last_seen) 
         VALUES ($1, $2, $3::pg_lsn, now())
         ON CONFLICT (node_id) 
         DO UPDATE SET last_lsn = $3::pg_lsn, last_seen = now()",
            &[&node_id, &slot_name, &lsn],
        )
        .await?;
    Ok(())
}
