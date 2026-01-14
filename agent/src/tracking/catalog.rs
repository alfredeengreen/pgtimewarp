use crate::store::StoreManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct TrackedRelation {
    pub relid: Option<u32>,
    pub pk_cols: Vec<String>,
    pub pk_strategy: i16,
}

pub struct Catalog {
    store: Arc<StoreManager>,
    node_id: String,
    relations: HashMap<(String, String), TrackedRelation>,
    source_client: Option<tokio_postgres::Client>,
    source_dsn: Option<String>,
}

impl Catalog {
    pub async fn new(store: Arc<StoreManager>, node_id: String) -> Result<Self> {
        let mut catalog = Self {
            store,
            node_id,
            relations: HashMap::new(),
            source_client: None,
            source_dsn: None,
        };

        catalog.refresh().await?;
        Ok(catalog)
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let client = self.store.client().await?;

        let rows = client
            .query(
                "SELECT schema_name, table_name, relid, pk_cols, pk_strategy 
             FROM pgtimewarp.tracked_relations 
             WHERE node_id = $1 AND status = 0",
                &[&self.node_id],
            )
            .await?;

        let mut new_relations = HashMap::new();

        for row in rows {
            let schema: String = row.get(0);
            let table: String = row.get(1);
            let relid: Option<i32> = row.get(2);
            let pk_cols: Vec<String> = row.get(3);
            let pk_strategy: i16 = row.get(4);

            new_relations.insert(
                (schema.clone(), table.clone()),
                TrackedRelation {
                    relid: relid.map(|r| r as u32),
                    pk_cols,
                    pk_strategy,
                },
            );
        }

        self.relations = new_relations;
        Ok(())
    }

    pub fn is_tracked(&self, schema: &str, table: &str) -> bool {
        self.relations
            .contains_key(&(schema.to_string(), table.to_string()))
    }

    pub fn get_relid(&self, schema: &str, table: &str) -> Option<u32> {
        self.relations
            .get(&(schema.to_string(), table.to_string()))
            .and_then(|r| r.relid)
    }

    pub fn get_pk_cols(&self, schema: &str, table: &str) -> Option<Vec<String>> {
        self.relations
            .get(&(schema.to_string(), table.to_string()))
            .map(|r| r.pk_cols.clone())
    }
}
