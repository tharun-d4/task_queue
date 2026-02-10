use sqlx::postgres::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::db::queries::update_heartbeat;

pub async fn start_heartbeat_task(
    heartbeat: u64,
    worker_id: Uuid,
    pool: PgPool,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(heartbeat));
        loop {
            interval.tick().await;

            if let Err(e) = update_heartbeat(&pool, worker_id).await {
                error!(worker_id = %worker_id, error = %e, "Heartbeat failed");
            } else {
                info!(worker_id = %worker_id, "Heartbeat sent");
            }
        }
    })
}
