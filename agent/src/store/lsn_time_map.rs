use anyhow::Result;
use tokio_postgres::Client;

pub async fn write_mapping(
    client: &Client,
    node_id: &str,
    lsn: &str,
) -> Result<()> {
    client.execute(
        "INSERT INTO pgtimewarp.lsn_time_map (node_id, ts, lsn) 
         VALUES ($1, now(), $2::pg_lsn)
         ON CONFLICT (node_id, ts) 
         DO UPDATE SET lsn = $2::pg_lsn",
        &[&node_id, &lsn],
    ).await?;
    Ok(())
}
