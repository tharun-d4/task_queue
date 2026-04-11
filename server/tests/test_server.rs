use axum_test::TestServer;
use server::{app, state};
use sqlx::PgPool;

pub fn build_test_server(pool: PgPool) -> TestServer {
    let state = state::AppState::new(pool);
    let app = app::create_router(state);

    TestServer::new(app).unwrap()
}
