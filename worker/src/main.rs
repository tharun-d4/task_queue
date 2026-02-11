use tracing::{error, info, instrument};
use uuid::Uuid;

use shared::{config::load_config, db::connection, tracing::init_tracing};
use worker::{db::queries, error::WorkerError, executor, handlers::email, heartbeat};

#[instrument]
#[tokio::main]
async fn main() -> Result<(), WorkerError> {
    let _trace_guard = init_tracing("worker");
    let config = load_config("./config").expect("Config Error");
    let pool = connection::create_pool(&config.database).await?;

    let worker_id = Uuid::now_v7();
    let pid = std::process::id();

    queries::register(&pool, worker_id, pid as i32).await?;
    info!(
        "[+] Worker (ID: {:?}, PID: {}) has started running & registered itself",
        worker_id, pid
    );

    heartbeat::start_heartbeat_task(pool.clone(), worker_id, config.worker.heartbeat).await;

    let smtp_sender = email::smtp_sender(&config.mail_server.host, config.mail_server.port);
    let client = reqwest::Client::new();

    loop {
        let claim_result = queries::claim_job(&pool, worker_id, config.worker.lease_duration).await;
        match claim_result {
            Ok(job) => {
                executor::execute_job(&pool, job, smtp_sender.clone(), client.clone()).await?;
            }
            Err(err) => {
                error!("Error occurred while fetching new job: {:?}", err);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
