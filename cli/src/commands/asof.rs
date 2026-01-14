use crate::store::Store;
use anyhow::Result;
use chrono::DateTime;
use serde_json::json;

pub async fn run(store: &Store, table: &str, pk: &str, at: &str, node: &str) -> Result<()> {
    let parts: Vec<&str> = table.split('.').collect();
    if parts.len() != 2 {
        anyhow::bail!("table must be in format schema.table");
    }

    let schema_name = parts[0];
    let table_name = parts[1];
    let at_ts: DateTime<chrono::Utc> = at.parse()?;

    let client = store.client();

    let row = client
        .query_opt(
            "SELECT rv.row_data, rv.valid_from_ts, rv.valid_from_lsn 
         FROM pgtimewarp.row_versions rv
         JOIN pgtimewarp.tracked_relations tr 
           ON rv.node_id = tr.node_id AND rv.relid = tr.relid
         WHERE tr.node_id = $1 
           AND tr.schema_name = $2 
           AND tr.table_name = $3
           AND rv.valid_from_lsn <= (
               SELECT lsn FROM pgtimewarp.lsn_time_map 
               WHERE node_id = $1 AND ts <= $4 
               ORDER BY ts DESC LIMIT 1
           )::pg_lsn
           AND (rv.valid_to_lsn IS NULL OR rv.valid_to_lsn > (
               SELECT lsn FROM pgtimewarp.lsn_time_map 
               WHERE node_id = $1 AND ts <= $4 
               ORDER BY ts DESC LIMIT 1
           )::pg_lsn)
         ORDER BY rv.valid_from_lsn DESC 
         LIMIT 1",
            &[&node, &schema_name, &table_name, &at_ts],
        )
        .await?;

    if let Some(row) = row {
        let row_data: Option<serde_json::Value> = row.get(0);
        let valid_from_ts: DateTime<chrono::Utc> = row.get(1);
        let valid_from_lsn: String = row.get(2);

        let result = json!({
            "row": row_data,
            "effective_as_of_ts": valid_from_ts,
            "effective_as_of_lsn": valid_from_lsn,
        });

        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("no row found at {}", at);
    }

    Ok(())
}
