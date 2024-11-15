use crate::{database::*, routes::AppState};
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, Json};
use chrono::NaiveDateTime;
use common::{Country, Gender, Platform};
use moka::future::Cache;
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio_postgres::error::SqlState;
use tracing::instrument;

pub struct ReadCache(Cache<Params, Vec<PartialAdvertisement>>);

impl ReadCache {
    pub fn new() -> Self {
        Self(
            Cache::builder()
                .weigher(|_, val: &Vec<PartialAdvertisement>| val.len() as u32)
                .time_to_live(Duration::new(60, 0))
                .max_capacity(131072)
                .build(),
        )
    }
    async fn get_or_insert_async<E, F, Fut>(
        &self,
        key: Params,
        f: F,
    ) -> Result<Vec<PartialAdvertisement>, Arc<E>>
    where
        F: FnOnce(Params) -> Fut,
        Fut: Future<Output = Result<Vec<PartialAdvertisement>, E>>,
        E: Send + Sync + 'static,
    {
        self.0.try_get_with(key.clone(), f(key)).await
    }
}

fn default_limit() -> usize {
    1
}

#[derive(serde::Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Params {
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    age: Option<i32>,
    #[serde(default)]
    country: Option<Country>,
    #[serde(default)]
    platform: Option<Platform>,
    #[serde(default)]
    gender: Option<Gender>,
}

#[derive(Serialize, Clone)]
pub struct PartialAdvertisement {
    title: String,
    end_at: NaiveDateTime,
}
#[derive(Serialize, Default, Clone)]
pub struct PartialAdvertisements {
    items: Vec<PartialAdvertisement>,
}

#[instrument(name = "GET /ad", skip(state, params))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<Params>,
) -> Result<Json<PartialAdvertisements>, StatusCode> {
    if params.limit == 0 {
        return Ok(Json(PartialAdvertisements::default()));
    }

    let client = &state.client;
    let items: Result<_, Arc<tokio_postgres::Error>> = state
        .read_cache
        .get_or_insert_async(params, move |params| async move {
            let ads = client
                .query_partial(
                    Condition {
                        age: params.age,
                        country: params.country,
                        platform: params.platform,
                        gender: params.gender
                    },
                    (params.limit, params.offset),
                )
                .await?;

            Ok(ads
                .into_iter()
                .map(|x| PartialAdvertisement {
                    title: x.title,
                    end_at: x.end_at,
                })
                .collect())
        })
        .await;

    let items = match items {
        Ok(ads) => ads,
        Err(err) => {
            tracing::error!("failed to query partial advertisements: {:?}", err);
            let code = SqlState::from_code("26000");
            if err.code()==Some(&code){
                std::process::exit(1);
            }
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(PartialAdvertisements { items }))
}
