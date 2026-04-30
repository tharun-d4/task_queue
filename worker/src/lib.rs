//! # Worker
//!
//! Worker is a process that executes the jobs claimed from the database
//! and stores the result or error back in the database.
//!
//! These jobs are initially set to pending state by the server that submits/stores the jobs initially
//! in the database when a user submits it. Then worker processes that are idle pick up the jobs
//! one at a time and matches with the current job types.
//!
//! Registered job types are:
//!     - send_email
//!     - send_webhook
//!     - will_crash (for testing purposes)
//!     - long_running_job (it just simulates a long running job)
//!
//! If the job type does not match with the registered job types, then that job is marked as
//! permanently failed since it is an invalid job type and there is no point in retrying it.
//!
//! If the job type matches, then the worker simply executes it. If the worker encountered an error,
//! then the error is stored in database.
//! If the error is permanent like a serialization error or a wrong payload structure error,
//! then the job is marked as permanently failed.
//!
//! If its a temporary error like a service unavailable for a webhook to be sent, then the job is marked
//! as pending so that another (or possibly the same) worker picks it up to retry it. The job is retried
//! until it succeeds and it satisfies the condition of attempts < max_retries. Also the retries
//! are done with exponential backoff in seconds.
//!
//! Note that the job has to be executed within the lease duration time, if execution time exceeds
//! this duraiton that job is automatically recovered by the server that assumes that the worker had
//! failed to run the job.
//!

pub mod background;
pub mod db;
pub mod error;
pub mod executor;
pub mod handlers;
pub mod prometheus;
pub mod state;

use std::{sync::Arc, time::Instant};

use shared::{config::load_worker_config, db::connection, tracing::init_tracing};
use tokio::signal::unix::{SignalKind, signal};
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{
    db::queries, error::WorkerError, handlers::email, prometheus::register_metrics, state::AppState,
};

#[instrument]
pub async fn init() -> Result<(), WorkerError> {
    let mut exit_when_no_jobs = false;
    let args = std::env::args().collect::<Vec<String>>();
    if let Some(s) = args.get(1)
        && s == "exit_when_no_jobs"
    {
        exit_when_no_jobs = true;
    }

    let _trace_guard = init_tracing("worker");
    let config = load_worker_config("./config")
        .map_err(|e| WorkerError::permanent("Failed to load worker config").set_source(e))?;

    let (registry, metrics) = register_metrics();
    let smtp_sender = email::smtp_sender(&config.mail_server.host, config.mail_server.port);
    let client = reqwest::Client::new();

    let state = Arc::new(AppState {
        registry: Arc::new(registry),
        metrics: Arc::new(metrics),
        client,
        smtp_sender,
    });

    let pool = connection::create_pool(config.database, config.worker.db_pool_size)
        .await
        .map_err(|e| WorkerError::permanent("Failed to establish db connection").set_source(e))?;
    connection::run_migrations(&pool)
        .await
        .map_err(|e| WorkerError::permanent("Failed to run db migrations").set_source(e))?;

    let worker_id = Uuid::now_v7();
    let pid = std::process::id();

    queries::register(&pool, worker_id, pid as i32).await?;
    info!(
        "Worker (ID: {:?}, PID: {}) has started running & registered itself",
        worker_id, pid
    );

    background::heartbeat_task(pool.clone(), worker_id, config.worker.heartbeat).await;
    background::push_metrics_task(
        state.clone(),
        worker_id,
        config.worker.metrics_push_interval,
    )
    .await;

    let mut terminate_signal = signal(SignalKind::terminate())
        .map_err(|e| WorkerError::permanent("Failed to create a SIGTERM listener").set_source(e))?;
    let mut iterrupt_signal = signal(SignalKind::interrupt())
        .map_err(|e| WorkerError::permanent("Failed to create a SIGINT listener").set_source(e))?;

    let mut job_count = 0_u32;
    let start = Instant::now();
    loop {
        tokio::select! {
            _ = terminate_signal.recv() => {
                info!("Received Terminate Signal(SIGTERM)");
                break;
            }
            _ = iterrupt_signal.recv() => {
                info!("Received Interrupt Signal(SIGINT)");
                break;
            }
            claim_result = queries::claim_job(&pool, worker_id, config.worker.lease_duration) => {
                match claim_result {
                    Ok(Some(job)) => {
                        job_count += 1;
                        let job_id = job.id;
                        let state_clone = state.clone();
                        let pool_clone = pool.clone();

                        tokio::spawn(async move {
                            if let Err(err) = executor::execute_job(pool_clone, state_clone, job, worker_id, job_id).await {
                                error!(error = ?err, "Got an error from executor");
                            };

                        });
                    }
                    Ok(None) => {
                        // No job to run
                        if exit_when_no_jobs {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                    Err(err) => error!(error = ?err, "Claim job error"),
                }
            }
        }
    }

    let end = start.elapsed();
    info!("== Processing results ==");
    info!("Processing time: {:.2}sec", end.as_secs_f64());
    info!(
        "Processing Rate: {:.2} jobs/sec",
        job_count as f64 / end.as_secs_f64()
    );

    info!("Worker (ID: {:?}, PID: {}) shutting down", worker_id, pid);
    queries::update_worker_shutdown_time(&pool, worker_id).await?;

    Ok(())
}
