use crate::config::SourceConfig;
use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls};

pub struct SlotManager {
    config: SourceConfig,
}

impl SlotManager {
    pub fn new(config: &SourceConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    pub async fn ensure_slot(&self) -> Result<()> {
        let (client, connection) = tokio_postgres::connect(&self.config.dsn, NoTls)
            .await
            .context("failed to connect to source database")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let slot_exists = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM pg_replication_slots WHERE slot_name = $1)",
                &[&self.config.slot_name],
            )
            .await
            .context("failed to check replication slot")?;

        let exists: bool = slot_exists.get(0);

        if !exists {
            let mut slot_options = Vec::new();

            if self.config.plugin == "wal2json" {
                if let Some(opts) = &self.config.wal2json_options {
                    if opts.include_lsn {
                        slot_options.push("'include-lsn' 'on'");
                    }
                    if opts.include_timestamp {
                        slot_options.push("'include-timestamp' 'on'");
                    }
                    if opts.include_typmod {
                        slot_options.push("'include-typmod' 'on'");
                    }
                    if opts.include_pk {
                        slot_options.push("'include-pk' 'on'");
                    }
                    if opts.pretty_print {
                        slot_options.push("'pretty-print' 'on'");
                    }
                    if opts.write_in_chunks {
                        slot_options.push("'write-in-chunks' 'on'");
                    }
                    if opts.include_old {
                        slot_options.push("'include-old' 'on'");
                    }
                }
            }

            let options_str = if slot_options.is_empty() {
                String::new()
            } else {
                format!("WITH ({})", slot_options.join(", "))
            };

            let query = format!(
                "SELECT pg_create_logical_replication_slot($1, $2 {})",
                options_str
            );

            client
                .execute(&query, &[&self.config.slot_name, &self.config.plugin])
                .await
                .context("failed to create replication slot")?;
        }

        Ok(())
    }

    pub async fn get_last_lsn(&self, store_dsn: &str) -> Result<Option<String>> {
        let (client, connection) = tokio_postgres::connect(store_dsn, NoTls)
            .await
            .context("failed to connect to store database")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let row = client
            .query_opt(
                "SELECT last_lsn FROM pgtimewarp.wal_checkpoints WHERE node_id = $1",
                &[&self.config.slot_name],
            )
            .await
            .context("failed to query last LSN")?;

        if let Some(row) = row {
            let lsn: Option<String> = row.get(0);
            Ok(lsn)
        } else {
            Ok(None)
        }
    }
}
