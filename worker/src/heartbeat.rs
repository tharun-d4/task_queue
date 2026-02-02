use sqlx::postgres::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::db::queries::heartbeat;

const HEARTBEAT_INTERVAL_SECS: u64 = 30;

pub async fn start_heartbeat_task(pool: PgPool, worker_id: Uuid) -> tokio::task::JoinHandle<()> {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

    interval.tick().await;

    tokio::spawn(async move {
        loop {
            interval.tick().await;

            if let Err(e) = heartbeat(&pool, worker_id).await {
                error!(worker_id = %worker_id, error = %e, "Heartbeat failed");
            } else {
                info!(worker_id = %worker_id, "Heartbeat sent");
            }
        }
    })
}
