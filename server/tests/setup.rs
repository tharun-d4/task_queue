use axum_test::TestServer;
use sqlx::PgPool;

use server::{app, state};
use shared::{config::load_test_config, db::connection};

pub struct TestFixture {
    pub server: TestServer,
    pub db: PgPool,
}

impl TestFixture {
    pub async fn new() -> Self {
        let config = load_test_config().expect("Test Config Error");

        let pool = connection::create_pool(&config.database).await.unwrap();
        connection::run_migrations(&pool).await.unwrap();

        let state = state::AppState::new(pool.clone());
        let app = app::create_router(state);
        let server = TestServer::new(app).unwrap();

        Self { server, db: pool }
    }
}
