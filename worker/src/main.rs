use tokio::signal::unix::{SignalKind, signal};
use tracing::{error, info, instrument};
use uuid::Uuid;

use shared::{config::load_worker_config, db::connection, tracing::init_tracing};
use worker::{db::queries, error::WorkerErrorV2, executor, handlers::email, heartbeat};

#[instrument]
#[tokio::main]
async fn main() -> Result<(), WorkerErrorV2> {
    let _trace_guard = init_tracing("worker");
    let config = load_worker_config("./config").expect("Config Error");

    let pool = connection::create_pool(&config.database).await.unwrap();
    connection::run_migrations(&pool).await.unwrap();

    let worker_id = Uuid::now_v7();
    let pid = std::process::id();

    queries::register(&pool, worker_id, pid as i32).await?;
    info!(
        "Worker (ID: {:?}, PID: {}) has started running & registered itself",
        worker_id, pid
    );

    heartbeat::start_heartbeat_task(pool.clone(), worker_id, config.worker.heartbeat).await;

    let smtp_sender = email::smtp_sender(&config.mail_server.host, config.mail_server.port);
    let client = reqwest::Client::new();

    let mut terminate_signal = signal(SignalKind::terminate()).unwrap();
    let mut iterrupt_signal = signal(SignalKind::interrupt()).unwrap();

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
                        executor::execute_job(&pool, job, worker_id, smtp_sender.clone(), client.clone()).await.unwrap();
                    }
                    Ok(None) => {
                        // No job to run
                        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    }
                    Err(err) => error!(error = ?err, "Claim job error"),
                }
            }
        }
    }
    info!("Worker (ID: {:?}, PID: {}) shutting down", worker_id, pid);
    queries::update_worker_shutdown_time(&pool, worker_id).await?;

    Ok(())
}
