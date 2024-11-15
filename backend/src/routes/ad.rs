use crate::{database::*, routes::AppState};
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, Json};
use chrono::NaiveDateTime;
use common::{Country, Gender, Platform};
use quick_cache::sync::Cache;
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;
use quick_cache::Weighter;
use tracing::instrument;


#[derive(Clone)]
struct VecWeighter;

impl Weighter<Params, Vec<PartialAdvertisement>> for VecWeighter{
    fn weight(&self, _: &Params, val: &Vec<PartialAdvertisement>) -> u64 {
        val.len() as u64
    }
}

pub struct ReadCache(Cache<Params, Vec<PartialAdvertisement>, VecWeighter>);

impl ReadCache {
    pub fn new() -> Self {
        Self(Cache::with_weighter(64, 512, VecWeighter))
    }
    async fn get_or_insert_async<E, F, Fut>(
        &self,
        key: Params,
        f: F,
    ) -> Result<Vec<PartialAdvertisement>, E>
    where
        F: FnOnce(Params) -> Fut,
        Fut: Future<Output = Result<Vec<PartialAdvertisement>, E>>,
    {
        if let Some(items) = self.0.get(&key) {
            return Ok(items);
        }
        f(key.clone()).await.map(|items| {
            self.0.insert(key, items.clone());
            items
        })
    }
}

fn default_limit() -> usize {
    1
}

#[derive(serde::Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
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
    let items: Result<_, tokio_postgres::Error> = state
        .read_cache
        .get_or_insert_async(params, move |params| async move {
            let ads = client
                .query_partial(
                    Condition {
                        age: params.age,
                        country: params.country,
                        platform: params.platform,
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
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if items.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(PartialAdvertisements { items }))
}
