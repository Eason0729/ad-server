//! domain routes
mod ad;
mod admin;
mod health;

use crate::database::Client;
use axum::{routing, Router};
use std::sync::Arc;

struct AppState {
    pub client: Client,
}

impl AppState {
    async fn new() -> Self {
        Self {
            client: Client::new().await,
        }
    }
    async fn shared() -> Arc<Self> {
        Arc::new(Self::new().await)
    }
}

pub async fn get_router() -> Router {
    Router::new()
        .route("/health", routing::get(health::handler))
        .route("/ad", routing::get(ad::handler))
        .route("/ad", routing::post(admin::handler))
        .with_state(AppState::shared().await)
}
