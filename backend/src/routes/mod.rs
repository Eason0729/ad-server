//! domain routes
mod ad;
mod admin;
mod health;

use crate::advertisement::PreparedClient;
use axum::{routing, Router};
use std::sync::Arc;

struct AppState {
    pub client: PreparedClient,
}

impl AppState {
    async fn new() -> Self {
        Self {
            client: PreparedClient::new_with_config(Default::default())
                .await
                .unwrap(),
        }
    }
    async fn shared() -> Arc<Self> {
        Arc::new(Self::new().await)
    }
}

pub async fn get_router() -> Router {
    Router::new()
        .route("/health", routing::get(health::handler))
        .route("/ads", routing::post(ad::handler))
        .route("/ad", routing::post(admin::handler))
        .with_state(AppState::shared().await)
}
