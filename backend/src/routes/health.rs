use crate::routes::AppState;
use axum::extract::State;
use std::sync::Arc;

pub async fn handler(_: State<Arc<AppState>>) -> &'static str {
    "OK"
}
