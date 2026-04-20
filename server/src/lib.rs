//! # Server
//! A HTTP Server through which jobs are
//! submitted, scheduled, monitored and managed
//! in the Job Scheduler project.
//!
//! Jobs are submited via REST API that are stored
//! in the database so that the worker processes
//! that run/execute the jobs fetch them from
//! the database table in which they are inserted.

pub mod app;
pub mod background;
pub mod db;
pub mod error;
pub mod handlers;
pub mod helper;
pub mod prometheus;
pub mod state;
pub mod utils;

use std::sync::Arc;

use shared::{config::load_server_config, db::connection, tracing::init_tracing};
use tracing::{info, instrument};

use crate::error::ServerError;

/// fn init() is the actual fn that setups the server.
///
/// The server loads its configuration from a centralized config.yaml file.
/// server:
///     host:
///     port:
///     db_pool_size:
///     ...
///
/// It connects to the database and creates a connection pool that is used
/// by the request handlers to perform database operations.
///
/// It runs the axum server at the host and port defined in the config file.
///
/// Prior to listening to the requests,
/// the server spawns three specific periodic tasks: Rescheduling, Lease recovery and Cleanup.
///
/// Rescheduling: Recurring jobs that have completed so far and were not rescheduled will be
/// rescheduled by this periodic task.
///
/// Lease Recovery Task: Jobs are leased to workers for a specific period of time
/// (that too is as per the config file).
/// If the jobs are executed within this time, then that's all well and good.
/// Otherwise, the jobs are assumed to be stalled or stuck due to a worker crash
/// or some other reason, and they are recovered so they can be retried
/// by another worker if the failure was temporary and not a permanent error.
///
/// Cleanup Task: It marks the jobs as permanently failed if they couldn't be marked
/// as such due to worker panicking while executing the jobs.
///
#[instrument]
pub async fn init() -> Result<(), error::ServerError> {
    let _trace_guard = init_tracing("server");
    let config = load_server_config("./config").expect("Config Error");

    let (registry, metrics) = prometheus::register_metrics();

    let pool = connection::create_pool(config.database, config.server.db_pool_size).await?;
    connection::run_migrations(&pool).await?;

    let state = Arc::new(state::AppState::new(pool.clone(), registry, metrics));

    background::lease_recovery_task(pool.clone(), state.clone(), config.server.lease_recovery)
        .await;
    background::rescheduling_recurring_jobs_task(
        pool.clone(),
        state.clone(),
        config.server.reschedule,
    )
    .await;
    background::cleanup_task(pool, config.server.cleanup).await;

    let app = app::create_router(state);

    let bind = format!("{}:{}", config.server.host, config.server.port);
    info!("[+] Server running on {bind:?}...");

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    Ok(())
}
