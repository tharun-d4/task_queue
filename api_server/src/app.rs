use std::sync::Arc;

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .with_state(Arc::new(state))
}
