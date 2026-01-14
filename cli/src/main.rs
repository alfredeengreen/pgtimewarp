use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod output;
mod store;

use config::Config;
use store::Store;

#[derive(Parser)]
#[command(name = "pgtimewarp")]
#[command(about = "PostgreSQL time travel management CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, env = "PGTIMEWARP_STORE_DSN")]
    store_dsn: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Doctor {
        #[arg(short, long)]
        node: Option<String>,
    },
    Track {
        table: String,
        #[arg(short, long)]
        pk: String,
        #[arg(short, long, default_value = "24")]
        retention: u32,
        #[arg(short, long)]
        node: String,
    },
    Untrack {
        table: String,
        #[arg(short, long)]
        node: String,
    },
    Status {
        #[arg(short, long)]
        node: Option<String>,
    },
    AsOf {
        table: String,
        #[arg(short, long)]
        pk: String,
        #[arg(short, long)]
        at: String,
        #[arg(short, long)]
        node: String,
    },
    Diff {
        table: String,
        #[arg(short, long)]
        pk: String,
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        node: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::load(cli.store_dsn)?;
    let store = Store::new(&config.store_dsn).await?;

    match cli.command {
        Commands::Doctor { node } => {
            commands::doctor::run(&store, node.as_deref()).await?;
        }
        Commands::Track {
            table,
            pk,
            retention,
            node,
        } => {
            commands::track::run(&store, &table, &pk, retention, &node).await?;
        }
        Commands::Untrack { table, node } => {
            commands::untrack::run(&store, &table, &node).await?;
        }
        Commands::Status { node } => {
            commands::status::run(&store, node.as_deref()).await?;
        }
        Commands::AsOf {
            table,
            pk,
            at,
            node,
        } => {
            commands::asof::run(&store, &table, &pk, &at, &node).await?;
        }
        Commands::Diff {
            table,
            pk,
            from,
            to,
            node,
        } => {
            commands::diff::run(&store, &table, &pk, &from, &to, &node).await?;
        }
    }

    Ok(())
}
