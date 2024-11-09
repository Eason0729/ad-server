use crate::{advertisement::Advertisement as AdvertisementModel, routes::AppState};
use axum::{extract::State, http::StatusCode, Json};
use common::{Country, Gender, Platform};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Advertisement {
    title: String,
    from_age: i32,
    to_age: i32,
    country: Option<Country>,
    end_at: chrono::NaiveDateTime,
    gender: Option<Gender>,
    platform: Option<Platform>,
}

impl From<Advertisement> for AdvertisementModel {
    fn from(value: Advertisement) -> Self {
        Self {
            title: value.title,
            age_range: (value.from_age, value.to_age),
            country: value.country,
            platform: value.platform,
            end_at: value.end_at,
        }
    }
}

#[axum::debug_handler]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(params): Json<Advertisement>,
) -> Result<(), StatusCode> {
    // TODO!: add log
    state.client.insert(&params.into()).await.unwrap();

    Ok(())
}
