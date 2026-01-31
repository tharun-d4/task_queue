use tracing::{info, instrument};
use uuid::Uuid;

use shared::{config::load_config, db::connection, tracing::init_tracing};
use worker::{db::queries, executor};

async fn sleep(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

#[instrument]
#[tokio::main]
async fn main() {
    let _trace_guard = init_tracing("worker");
    let config = load_config("./config").expect("Config Error");
    let pool = connection::create_pool(&config.database).await.unwrap();

    let worker_id = Uuid::now_v7();
    let pid = std::process::id();

    queries::register(&pool, worker_id, pid as i32)
        .await
        .unwrap();
    info!(
        "[+] Worker (ID: {:?}, PID: {}) has started running & registered itself",
        worker_id, pid
    );

    loop {
        let job = queries::claim_job(&pool, worker_id).await.unwrap();
        let job_id = job.id.clone();

        match executor::execute_job(job).await {
            Ok(result) => {
                queries::mark_job_as_completed(&pool, job_id, result)
                    .await
                    .unwrap();
            }
            Err(_) => {}
        }
        sleep(10000).await;
    }
}
