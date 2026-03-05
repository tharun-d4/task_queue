use tracing::{info, instrument};

use server::{app, background, error, state};
use shared::{config::load_server_config, db::connection, tracing::init_tracing};

/// Server entrypoint.
///
/// Responsible for:
/// - Initializing tracing/logging
/// - Loading configuration
/// - Establishing a database connection
/// - Running database migrations
/// - Spawning background maintenance tasks
/// - Starting the HTTP server
///
/// Background tasks (lease recovery, cleanup) are started before starting the server.

#[instrument]
#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    let _trace_guard = init_tracing("server");
    let config = load_server_config("./config").expect("Config Error");

    let pool = connection::create_pool(&config.database).await?;
    connection::run_migrations(&pool).await?;

    background::lease_recovery_task(pool.clone(), config.server.lease_recovery).await;
    background::cleanup_task(pool.clone(), config.server.cleanup).await;

    let state = state::AppState::new(pool);
    let app = app::create_router(state);

    let bind = format!("{}:{}", config.server.host, config.server.port);
    info!("[+] Server running on {bind:?}...");

    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
