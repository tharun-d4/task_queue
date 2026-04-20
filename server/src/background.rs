use std::sync::Arc;

use sqlx::postgres::PgPool;
use tracing::{error, warn};

use crate::{db::queries, error::ServerError, helper, prometheus::JobType, state::AppState};

pub async fn lease_recovery_task(
    pool: PgPool,
    state: Arc<AppState>,
    recovery_interval: u8,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(recovery_interval as u64));

        loop {
            interval.tick().await;

            let result = queries::recover_lease_expired_jobs(&pool).await;
            match result {
                Ok(jobs) => {
                    if !jobs.is_empty() {
                        let recovered: i64 = jobs.iter().map(|row| row.count).sum();
                        warn!("Jobs Recovered: {}", recovered);

                        for job in jobs {
                            state
                                .metrics
                                .lease_recovered_jobs
                                .get_or_create(&JobType {
                                    job_type: job.job_type,
                                })
                                .inc_by(job.count as u64);
                        }
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

            let result = queries::mark_retry_exhausted_jobs_as_failed(&pool).await;
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

pub async fn rescheduling_recurring_jobs_task(
    pool: PgPool,
    state: Arc<AppState>,
    interval: u8,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let state = state;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval as u64));

        loop {
            interval.tick().await;

            if let Err(ServerError::Database(err)) =
                helper::reschedule_recurring_jobs(&pool, &state).await
            {
                error!(error = ?err, "Error occurred while rescheduling recurring jobs");
            }
        }
    })
}
