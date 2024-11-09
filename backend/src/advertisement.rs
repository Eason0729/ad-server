//! persistence layer
//!

use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use chrono::NaiveDateTime;
use common::{Country, Platform};
use std::fmt::format;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, NoTls};

pub type Manager = PostgresConnectionManager<NoTls>;
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
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
            password: "password".to_string(),
            db: "postgres".to_string(),
        }
    }
}

/// connection pool
pub struct PreparedClient {
    pool: Pool<Manager>,
    queries: Queries,
}

impl PreparedClient {
    pub async fn new_with_config(config: Config) -> Result<Self, tokio_postgres::Error> {
        let manager =
            PostgresConnectionManager::new_from_stringlike("host=localhost user=postgres", NoTls)?;
        let pool = Pool::builder().max_size(15).build(manager).await?;

        PreparedClient::new(pool).await
    }
    pub async fn new(
        pool: Pool<PostgresConnectionManager<NoTls>>,
    ) -> Result<Self, tokio_postgres::Error> {
        Ok(PreparedClient {
            queries: Queries::new(&pool).await?,
            pool,
        })
    }
    pub async fn insert(&self, advertisement: &Advertisement) -> Result<(), tokio_postgres::Error> {
        self.pool
            .get()
            .await
            .expect("Failed to get connection")
            .execute(
                &self.queries.insert_stmt,
                &[
                    &advertisement.title,
                    &advertisement.age_range.0,
                    &advertisement.age_range.1,
                    &advertisement
                        .country
                        .clone()
                        .map(Country::into_id)
                        .unwrap_or_default(),
                    &advertisement.platform.map(|p| p as u32).unwrap_or_default(),
                    &advertisement.end_at.format(TIME_FORMAT).to_string(),
                ],
            )
            .await?;
        Ok(())
    }
    pub async fn query_partial(
        &self,
        cond: Condition,
        (limit, offset): (usize, usize),
    ) -> Result<Vec<PartialAdvertisement>, tokio_postgres::Error> {
        let stmt = self.queries.get_query_stmt(
            cond.country.is_some(),
            cond.platform.is_some(),
            cond.age.is_some(),
        );
        let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();

        let country;
        if let Some(x) = cond.country {
            country = x.into_id();
            params.push(&country);
        }

        let platform;
        if let Some(x) = cond.platform {
            platform = x as u32;
            params.push(&platform);
        }

        let age;
        if let Some(x) = cond.age {
            age = x;
            params.push(&age);
        }

        let offset = &(offset as u32);
        let limit = &(limit as u32);
        params.push(offset);
        params.push(limit);

        let rows = self
            .pool
            .get()
            .await
            .expect("Fail to get connection")
            .query(stmt, &params)
            .await?;
        Ok(rows
            .iter()
            .map(|row| PartialAdvertisement {
                id: row.get(0),
                title: row.get(1),
                end_at: NaiveDateTime::parse_from_str(row.get(2), TIME_FORMAT).unwrap(),
            })
            .collect())
    }
}

pub struct Advertisement {
    pub title: String,
    pub age_range: (i32, i32), // int4range
    pub country: Option<Country>,
    pub platform: Option<Platform>,
    pub end_at: NaiveDateTime,
}

pub struct PartialAdvertisement {
    pub id: i32,
    pub title: String,
    pub end_at: NaiveDateTime,
}

pub struct Condition {
    pub age: Option<i32>,
    pub country: Option<Country>,
    pub platform: Option<Platform>,
}

struct Queries {
    insert_stmt: tokio_postgres::Statement,
    query_stmt: [tokio_postgres::Statement; 1 << 4],
}

impl Queries {
    pub async fn new(pool: &Pool<Manager>) -> Result<Self, tokio_postgres::Error> {
        let conn = pool.get().await.expect("Failed to get connection");
        let insert_stmt = conn.prepare("INSERT INTO advertisements (title, age_range, country, platform, end_at) VALUES ($1, int4range($2, $3), $4, $5, $6)").await?;
        let mut query_stmt = std::array::from_fn(|_| None);
        for i in 0..1 << 4 {
            let mut query = "SELECT id, title, end_at FROM advertisements".to_string();
            let mut filters = Vec::new();
            let mut n = 1;

            if i & 1 != 0 {
                filters.push(format!("country = ${}", n));
                n += 1;
            }
            if i & 2 != 0 {
                filters.push(format!("platform = ${}", n));
            }
            if i & 4 != 0 {
                filters.push(format!("age_range @> ${}", n));
            }
            if i & 8 != 0 {
                filters.push("end_at > now()".to_string());
            }
            if !filters.is_empty() {
                query.push_str(" WHERE ");
                query.push_str(&filters.join(" AND "));
            }
            query.push_str("ORDER BY id LIMIT $7, $8");
            query_stmt[i] = Some(conn.prepare(&query).await?);
        }
        let query_stmt = query_stmt.map(|stmt| stmt.unwrap());
        Ok(Queries {
            insert_stmt,
            query_stmt,
        })
    }
    fn get_query_stmt(
        &self,
        country: bool,
        platform: bool,
        age: bool,
    ) -> &tokio_postgres::Statement {
        let mut idx = 0;
        if country {
            idx |= 1;
        }
        if platform {
            idx |= 1 << 1;
        }
        if age {
            idx |= 1 << 2;
        }
        &self.query_stmt[idx]
    }
}
