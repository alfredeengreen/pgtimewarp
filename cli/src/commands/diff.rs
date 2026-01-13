use crate::store::Store;
use anyhow::Result;
use chrono::DateTime;
use serde_json::json;

pub async fn run(
    store: &Store,
    table: &str,
    pk: &str,
    from: &str,
    to: &str,
    node: &str,
) -> Result<()> {
    let parts: Vec<&str> = table.split('.').collect();
    if parts.len() != 2 {
        anyhow::bail!("table must be in format schema.table");
    }
    
    let schema_name = parts[0];
    let table_name = parts[1];
    let from_ts: DateTime<chrono::Utc> = from.parse()?;
    let to_ts: DateTime<chrono::Utc> = to.parse()?;
    
    let client = store.client();
    
    let rows = client.query(
        "SELECT rv.row_data, rv.valid_from_ts, rv.valid_from_lsn, rv.op 
         FROM pgtimewarp.row_versions rv
         JOIN pgtimewarp.tracked_relations tr 
           ON rv.node_id = tr.node_id AND rv.relid = tr.relid
         WHERE tr.node_id = $1 
           AND tr.schema_name = $2 
           AND tr.table_name = $3
           AND rv.valid_from_lsn >= (
               SELECT lsn FROM pgtimewarp.lsn_time_map 
               WHERE node_id = $1 AND ts <= $4 
               ORDER BY ts DESC LIMIT 1
           )::pg_lsn
           AND rv.valid_from_lsn <= (
               SELECT lsn FROM pgtimewarp.lsn_time_map 
               WHERE node_id = $1 AND ts <= $5 
               ORDER BY ts DESC LIMIT 1
           )::pg_lsn
         ORDER BY rv.valid_from_lsn",
        &[&node, &schema_name, &table_name, &from_ts, &to_ts],
    ).await?;
    
    let mut versions = Vec::new();
    for row in rows {
        let row_data: Option<serde_json::Value> = row.get(0);
        let valid_from_ts: DateTime<chrono::Utc> = row.get(1);
        let valid_from_lsn: String = row.get(2);
        let op: i16 = row.get(3);
        let op_str = match op {
            0 => "insert",
            1 => "update",
            2 => "delete",
            _ => "unknown",
        };
        
        versions.push(json!({
            "op": op_str,
            "row": row_data,
            "valid_from_ts": valid_from_ts,
            "valid_from_lsn": valid_from_lsn,
        }));
    }
    
    let result = json!({
        "versions": versions,
    });
    
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}
