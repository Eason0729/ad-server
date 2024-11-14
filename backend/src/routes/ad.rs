use crate::{database::*, routes::AppState};
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, Json};
use chrono::NaiveDateTime;
use common::{Country, Gender, Platform};
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;

fn default_limit() -> usize {
    1
}

#[derive(serde::Deserialize, Debug)]
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

#[derive(Serialize)]
pub struct PartialAdvertisement {
    title: String,
    end_at: NaiveDateTime,
}
#[derive(Serialize, Default)]
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

    let ads = match state
        .client
        .query_partial(
            Condition {
                age: params.age,
                country: params.country,
                platform: params.platform,
            },
            (params.limit, params.offset),
        )
        .await
    {
        Ok(ads) => ads,
        Err(err) => {
            tracing::error!("failed to query partial advertisements: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let items: Vec<_> = ads
        .into_iter()
        .map(|x| PartialAdvertisement {
            title: x.title,
            end_at: x.end_at,
        })
        .collect();

    if items.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(PartialAdvertisements { items }))
}
