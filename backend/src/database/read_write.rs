use crate::database::{Connection, Manager};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

static POOL_EXHAUSTED_MSG: &str = "cannot found/add new connection to pool";

pub struct Config {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub db: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "localhost".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            password: "postgres".to_string(),
            db: "postgres".to_string(),
        }
    }
}
impl Config {
    fn to_stringlike(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, self.user, self.password, self.db
        )
    }
    pub fn with_host(mut self, host: impl AsRef<str>) -> Self {
        self.host = host.as_ref().to_string();
        self
    }
    pub fn with_password(mut self, password: impl AsRef<str>) -> Self {
        self.password = password.as_ref().to_string();
        self
    }
}

/// connection pool
pub struct Client {
    read_pool: Pool<Manager>,
    write_pool: Pool<Manager>,
}

impl Client {
    pub async fn new_with_config(
        read: Config,
        write: Config,
    ) -> Result<Self, tokio_postgres::Error> {
        let read_manager =
            PostgresConnectionManager::new_from_stringlike(read.to_stringlike(), NoTls)?;
        let read_pool = Pool::builder().max_size(15).build(read_manager).await?;

        let write_manager =
            PostgresConnectionManager::new_from_stringlike(write.to_stringlike(), NoTls)?;
        let write_pool = Pool::builder().max_size(15).build(write_manager).await?;

        Client::new(read_pool, write_pool).await
    }
    pub async fn new(
        read: Pool<PostgresConnectionManager<NoTls>>,
        write: Pool<PostgresConnectionManager<NoTls>>,
    ) -> Result<Self, tokio_postgres::Error> {
        Ok(Client {
            read_pool: read,
            write_pool: write,
        })
    }
    pub async fn read(&self) -> Connection {
        self.read_pool.get().await.expect(POOL_EXHAUSTED_MSG)
    }
    pub async fn write(&self) -> Connection {
        self.write_pool.get().await.expect(POOL_EXHAUSTED_MSG)
    }
}
