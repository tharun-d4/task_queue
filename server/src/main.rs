use tracing::{info, instrument};

use server::{app, error, state};
use shared::{config::load_config, db::connection, tracing::init_tracing};

#[instrument]
#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    let _trace_guard = init_tracing("server");
    let config = load_config("./config").expect("Config Error");

    let pool = connection::create_pool(&config.database).await?;
    connection::run_migrations(&pool).await?;

    let state = state::AppState::new(pool);
    let app = app::create_router(state);

    let bind = format!("{}:{}", config.server.host, config.server.port);
    info!("[+] Server running on {bind:?}...");

    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
