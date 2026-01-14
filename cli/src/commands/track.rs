use crate::store::Store;
use anyhow::Result;

pub async fn run(store: &Store, table: &str, pk: &str, retention: u32, node: &str) -> Result<()> {
    let parts: Vec<&str> = table.split('.').collect();
    if parts.len() != 2 {
        anyhow::bail!("table must be in format schema.table");
    }

    let schema_name = parts[0];
    let table_name = parts[1];
    let pk_cols: Vec<String> = pk.split(',').map(|s| s.trim().to_string()).collect();

    let client = store.client();

    client.execute(
        "INSERT INTO pgtimewarp.tracked_relations 
         (node_id, schema_name, table_name, pk_cols, pk_strategy, replica_identity_full, status, retention_hours) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (node_id, schema_name, table_name) 
         DO UPDATE SET pk_cols = $4, retention_hours = $8, updated_at = now()",
        &[
            &node,
            &schema_name,
            &table_name,
            &pk_cols,
            &(if pk_cols.len() == 1 { 0i16 } else { 1i16 }),
            &true,
            &0i16,
            &(retention as i32),
        ],
    ).await?;

    println!(
        "tracking {}.{} with primary key: {}",
        schema_name, table_name, pk
    );

    Ok(())
}
