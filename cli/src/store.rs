use anyhow::Result;
use tokio_postgres::{Client, NoTls};

pub struct Store {
    client: Client,
}

impl Store {
    pub async fn new(dsn: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("store connection error: {}", e);
            }
        });

        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
