use std::sync::Arc;

use shared::db::models::Job;
use sqlx::postgres::PgPool;
use tokio::time::Instant;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{
    db::queries,
    error::WorkerError,
    handlers::{email::send_email, webhook::send_webhook},
    prometheus::JobType,
    state::AppState,
};

#[instrument(skip(pool, state))]
pub async fn execute_job(
    pool: &PgPool,
    state: Arc<AppState>,
    job: Job,
    worker_id: Uuid,
) -> Result<(), WorkerError> {
    let job_id = job.id;

    let retries_exhausted = job.attempts == job.max_retries;

    let backoff_secs = retry_backoff_secs(job.attempts);

    let start = Instant::now();
    let result = match job.job_type.as_ref() {
        "send_email" => send_email(state.smtp_sender.clone(), job.payload).await,
        "send_webhook" => send_webhook(state.client.clone(), job.payload).await,
        "will_crash" => {
            error!("Worker will crash when running this job");
            panic!("Worker crashed when running this job");
        }
        "long_running_job" => {
            info!("This is a long running job");
            // blocking sleep - mocking synchronous work
            std::thread::sleep(std::time::Duration::from_secs(10));
            Ok(None)
        }
        _ => Err(WorkerError::permanent("Invalid job type")),
    };
    let end = start.elapsed();
    info!(duration_ms = end.as_millis(), "Job executed");

    let job_type_clone = job.job_type.clone();

    match result {
        Ok(res) => {
            info!("Job completed");
            let moved_jobs = queries::mark_job_as_completed(pool, job_id, worker_id, res).await?;
            if moved_jobs != 1 {
                error!(moved_jobs = moved_jobs, "Failed to mark job as completed");
            } else {
                state
                    .metrics
                    .jobs_completed
                    .get_or_create(&JobType {
                        job_type: job.job_type,
                    })
                    .inc();
            }
        }
        Err(err) => {
            error!(
                error = ?err,
                retry_backoff_time_in_secs = backoff_secs,
                "Failed to execute job"
            );

            if err.is_permanent() || retries_exhausted {
                let moved_rows =
                    queries::mark_job_as_failed(pool, job_id, worker_id, err.to_string()).await?;
                if moved_rows != 1 {
                    error!(moved_rows = moved_rows, "Failed to job as failed");
                } else {
                    state
                        .metrics
                        .jobs_failed
                        .get_or_create(&JobType {
                            job_type: job.job_type,
                        })
                        .inc();
                }
            } else {
                let updated_rows = queries::update_job_error_and_backoff_time(
                    pool,
                    job_id,
                    worker_id,
                    err.to_string(),
                    backoff_secs,
                )
                .await?;
                if updated_rows != 1 {
                    error!(
                        updated_rows = updated_rows,
                        "Failed to update job error and retry time"
                    );
                } else {
                    state
                        .metrics
                        .jobs_retried
                        .get_or_create(&JobType {
                            job_type: job.job_type,
                        })
                        .inc();
                }
            }
        }
    }

    let end = start.elapsed();
    info!(overall_duration_ms = end.as_millis(), "Job updated in DB");

    state
        .metrics
        .job_processing_duration_seconds
        .get_or_create(&JobType {
            job_type: job_type_clone,
        })
        .observe(end.as_secs_f64());

    Ok(())
}

fn retry_backoff_secs(attempts: i16) -> i16 {
    2_i16.pow(attempts as u32)
}

#[cfg(test)]
mod tests {
    use super::retry_backoff_secs;

    #[test]
    fn backoff_secs_with_4_attempts_returns_16() {
        assert_eq!(retry_backoff_secs(4), 16);
    }

    #[test]
    fn backoff_secs_with_10_attempts_returns_1024() {
        assert_eq!(retry_backoff_secs(10), 1024);
    }
}
