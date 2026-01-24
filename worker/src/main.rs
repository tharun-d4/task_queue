use tracing::{info, instrument};
use uuid::Uuid;

use shared::{config::load_config, db::connection, tracing::init_tracing};

async fn sleep(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

#[instrument]
#[tokio::main]
async fn main() {
    let _trace_guard = init_tracing("worker");
    let config = load_config("../config").expect("Config Error");
    let pool = connection::create_pool(&config.database).await.unwrap();

    let worker_id = Uuid::now_v7();
    info!("[+] Worker - ID: {:?} has started running...", worker_id);

    loop {
        sleep(10000).await;
    }
}
