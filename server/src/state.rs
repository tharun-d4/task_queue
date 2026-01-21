use sqlx::postgres::PgPool;

#[derive(Debug)]
pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        AppState { pool }
    }
}
