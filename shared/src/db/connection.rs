use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config;

pub async fn create_pool(db_config: &config::Database) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(db_config.pool_size as u32)
        .connect(&db_config.url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../migrations").run(pool).await?;
    Ok(())
}
