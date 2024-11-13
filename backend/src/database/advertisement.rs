use crate::database::Connection;
use chrono::NaiveDateTime;
use common::{Country, Platform};
use std::time::SystemTime;
use tokio_postgres::types::{ToSql, Type};

pub const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub(crate) struct Queries {
    insert_stmt: tokio_postgres::Statement,
    query_stmt: [tokio_postgres::Statement; 1 << 4],
}

impl Queries {
    pub async fn new(
        read_conn: &Connection<'_>,
        write_conn: &Connection<'_>,
    ) -> Result<Self, tokio_postgres::Error> {
        let insert_stmt = write_conn
            .prepare_typed(
                r#"INSERT INTO advertisement (title, age_range, country, platform, end_at)
                VALUES ($1, Int4Range($2, $3), $4, $5, $6);"#,
                &[
                    Type::TEXT,
                    Type::INT4,
                    Type::INT4,
                    Type::INT4,
                    Type::INT4,
                    Type::TIMESTAMP,
                ],
            )
            .await?;

        let mut query_stmt = std::array::from_fn(|_| None);
        for i in 0..1 << 4 {
            let mut query = "SELECT id, title, end_at FROM advertisement".to_string();
            let mut filters = Vec::new();
            let mut n = 1;

            if i & 1 != 0 {
                filters.push(format!("country = ${}", n));
                n += 1;
            }
            if i & 2 != 0 {
                filters.push(format!("platform = ${}", n));
                n += 1;
            }
            if i & 4 != 0 {
                filters.push(format!("age_range @> ${}", n));
                n += 1;
            }
            if i & 8 != 0 {
                filters.push("end_at > now()".to_string());
            }
            if !filters.is_empty() {
                query.push_str(" WHERE ");
                query.push_str(&filters.join(" AND "));
            }
            query.push_str(format!(" ORDER BY id LIMIT ${} OFFSET ${}", n, n + 1).as_str());
            query_stmt[i] = Some(read_conn.prepare(&query).await?);
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

impl Queries {
    pub async fn insert(
        &self,
        advertisement: &Advertisement,
        write: &Connection<'_>,
    ) -> Result<(), tokio_postgres::Error> {
        write
            .execute(
                &self.insert_stmt,
                &[
                    &advertisement.title,
                    &(advertisement.age_range.0),
                    &(advertisement.age_range.1),
                    &advertisement
                        .country
                        .clone()
                        .map(|x| x.into_id() as i32)
                        .unwrap_or_default(),
                    &advertisement.platform.map(|p| p as i32).unwrap_or_default(),
                    &SystemTime::from(advertisement.end_at.and_utc()),
                ],
            )
            .await?;
        Ok(())
    }
    pub async fn query_partial(
        &self,
        read: &Connection<'_>,
        cond: Condition,
        (limit, offset): (usize, usize),
    ) -> Result<Vec<PartialAdvertisement>, tokio_postgres::Error> {
        let stmt = self.get_query_stmt(
            cond.country.is_some(),
            cond.platform.is_some(),
            cond.age.is_some(),
        );
        let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();

        let country;
        if let Some(x) = cond.country {
            country = x.into_id() as i64;
            params.push(&country);
        }

        let platform;
        if let Some(x) = cond.platform {
            platform = x as i64;
            params.push(&platform);
        }

        let age;
        if let Some(x) = cond.age {
            age = x as i64;
            params.push(&age);
        }

        let offset = &(offset as i64);
        let limit = &(limit as i64);
        params.push(offset);
        params.push(limit);

        let rows = read.query(stmt, &params).await?;
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
