use anyhow::{Context, Result};

pub struct Config {
    pub store_dsn: String,
}

impl Config {
    pub fn load(store_dsn: Option<String>) -> Result<Self> {
        let store_dsn = store_dsn
            .or_else(|| std::env::var("PGTIMEWARP_STORE_DSN").ok())
            .context("store_dsn required (--store-dsn or PGTIMEWARP_STORE_DSN)")?;
        
        Ok(Self { store_dsn })
    }
}
