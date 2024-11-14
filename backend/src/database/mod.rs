use crate::database::read_write::Config;
use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use std::env;
use tokio_postgres::NoTls;

pub mod advertisement;
pub mod read_write;

pub use advertisement::{Advertisement, Condition, PartialAdvertisement};

type Connection<'a> = PooledConnection<'a, Manager>;

type Manager = PostgresConnectionManager<NoTls>;

pub struct Client {
    inner_client: read_write::Client,
    queries: advertisement::Queries,
}

impl Client {
    pub async fn new() -> Self {
        let read_host = env::var("READ_HOST").unwrap_or("localhost".to_string());
        let write_host = env::var("WRITE_HOST").unwrap_or("localhost".to_string());

        let inner_client = read_write::Client::new_with_config(
            Config::default().with_host(read_host),
            Config::default().with_host(write_host),
        )
        .await
        .unwrap();
        let queries =
            advertisement::Queries::new(&inner_client.read().await, &inner_client.write().await)
                .await
                .unwrap();

        Self {
            inner_client,
            queries,
        }
    }
    pub async fn insert(&self, advertisement: &Advertisement) -> Result<(), tokio_postgres::Error> {
        self.queries
            .insert(advertisement, &self.inner_client.write().await)
            .await
    }
    pub async fn query_partial(
        &self,
        cond: Condition,
        (limit, offset): (usize, usize),
    ) -> Result<Vec<PartialAdvertisement>, tokio_postgres::Error> {
        self.queries
            .query_partial(&self.inner_client.read().await, cond, (limit, offset))
            .await
    }
}
