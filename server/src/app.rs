use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tower::ServiceBuilder;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;

use crate::{handlers::create_job, state::AppState};

pub fn create_router(state: AppState) -> Router {
    let middleware = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO))
            .on_failure(DefaultOnFailure::new().level(Level::INFO)),
    );

    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .route("/jobs", post(create_job))
        .with_state(Arc::new(state))
        .layer(middleware)
}
