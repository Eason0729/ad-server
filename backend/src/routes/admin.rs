use crate::{database::Advertisement as AdvertisementModel, routes::AppState};
use axum::{extract::State, http::StatusCode, Json};
use common::{Country, Gender, Platform};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
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

#[tracing::instrument(name = "POST /ad", skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(params): Json<Advertisement>,
) -> Result<(), StatusCode> {
    if let Err(err) = state.client.insert(&params.into()).await {
        tracing::error!("failed to insert advertisement: {:?}", err);
    }

    Ok(())
}
