use std::sync::Arc;

use axum_test::TestServer;
use server::{app, prometheus::register_metrics, state};
use sqlx::PgPool;

pub fn build_test_server(pool: PgPool) -> TestServer {
    let (registry, metrics) = register_metrics();
    let state = Arc::new(state::AppState::new(pool, registry, metrics));
    let app = app::create_router(state);

    TestServer::new(app).unwrap()
}

//datasources:
//  - name: Metrics
//    type: prometheus
//    access: proxy
//    url: http://prometheus:9090/
