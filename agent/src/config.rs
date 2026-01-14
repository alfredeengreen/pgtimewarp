use anyhow::{Context, Result};
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub node_id: String,
    pub agent_version: String,
    pub source: SourceConfig,
    pub store: StoreConfig,
    pub intervals: IntervalsConfig,
    pub limits: LimitsConfig,
    pub privacy: Option<PrivacyConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceConfig {
    pub dsn: String,
    pub slot_name: String,
    pub plugin: String,
    pub wal2json_options: Option<Wal2JsonOptions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Wal2JsonOptions {
    #[serde(default = "default_true")]
    pub include_lsn: bool,
    #[serde(default = "default_true")]
    pub include_timestamp: bool,
    #[serde(default = "default_false")]
    pub include_typmod: bool,
    #[serde(default = "default_true")]
    pub include_pk: bool,
    #[serde(default = "default_false")]
    pub pretty_print: bool,
    #[serde(default = "default_false")]
    pub write_in_chunks: bool,
    #[serde(default = "default_true")]
    pub include_old: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoreConfig {
    pub dsn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntervalsConfig {
    pub refresh_tracked_s: u64,
    pub retention_s: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    pub batch_size: usize,
    pub max_queue: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivacyConfig {
    pub allow_tables: Option<Vec<String>>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Figment::new()
            .merge(Yaml::file(path))
            .merge(Env::prefixed("PGTIMEWARP_"))
            .extract()
            .context("failed to load configuration")
    }
}

impl SourceConfig {
    pub fn default_wal2json_options() -> Wal2JsonOptions {
        Wal2JsonOptions {
            include_lsn: true,
            include_timestamp: true,
            include_typmod: false,
            include_pk: true,
            pretty_print: false,
            write_in_chunks: false,
            include_old: true,
        }
    }
}
