use sqlx::postgres::PgPool;
use tracing::{error, warn};

use crate::db::queries::{
    move_retry_exhausted_jobs_to_failed_jobs, recover_unfinished_lease_expired_jobs,
};

pub async fn lease_recovery_task(
    pool: PgPool,
    recovery_interval: u8,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(recovery_interval as u64));

        loop {
            interval.tick().await;

            let result = recover_unfinished_lease_expired_jobs(&pool).await;
            match result {
                Ok(count) => {
                    if count > 0 {
                        warn!("Jobs Recovered: {}", count);
                    }
                }
                Err(err) => {
                    error!(
                        error = ?err,
                        "Error occured while recovering unfinished and lease expired jobs: "
                    )
                }
            };
        }
    })
}

pub async fn cleanup_task(pool: PgPool, interval: u8) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval as u64));

        loop {
            interval.tick().await;

            let result = move_retry_exhausted_jobs_to_failed_jobs(&pool).await;
            match result {
                Ok(count) => {
                    if count > 0 {
                        warn!("Jobs Failed: {}", count);
                    }
                }
                Err(err) => {
                    error!(
                        error = ?err,
                        "Error occured while cleaning up retry-exhausted jobs: "
                    )
                }
            };
        }
    })
}
