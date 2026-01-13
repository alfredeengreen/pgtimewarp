use crate::store::Store;
use anyhow::Result;

pub async fn run(store: &Store, table: &str, node: &str) -> Result<()> {
    let parts: Vec<&str> = table.split('.').collect();
    if parts.len() != 2 {
        anyhow::bail!("table must be in format schema.table");
    }
    
    let schema_name = parts[0];
    let table_name = parts[1];
    
    let client = store.client();
    
    let deleted = client.execute(
        "DELETE FROM pgtimewarp.tracked_relations 
         WHERE node_id = $1 AND schema_name = $2 AND table_name = $3",
        &[&node, &schema_name, &table_name],
    ).await?;
    
    if deleted > 0 {
        println!("stopped tracking {}.{}", schema_name, table_name);
    } else {
        println!("{}.{} was not being tracked", schema_name, table_name);
    }
    
    Ok(())
}
