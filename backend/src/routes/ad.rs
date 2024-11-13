use crate::{database::*, routes::AppState};
use axum::{extract::State, http::StatusCode, Json};
use chrono::NaiveDateTime;
use common::{Country, Gender, Platform};
use serde::Serialize;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct Params {
    #[serde(default)]
    offset: usize,
    #[serde(default)]
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

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(params): Json<Params>,
) -> Result<Json<PartialAdvertisements>, StatusCode> {
    if params.limit == 0 {
        return Ok(Json(PartialAdvertisements::default()));
    }

    // TODO!: add log
    let ads = state
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
        .unwrap();

    let items:Vec<_> = ads
        .into_iter()
        .map(|x| PartialAdvertisement {
            title: x.title,
            end_at: x.end_at,
        })
        .collect();

    if items.is_empty(){
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(PartialAdvertisements { items }))
}
