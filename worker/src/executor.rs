use lettre::{Tokio1Executor, transport::smtp::AsyncSmtpTransport};
use sqlx::postgres::PgPool;
use tracing::{error, info, instrument};
use uuid::Uuid;

use shared::db::models::Job;

use crate::{
    db::queries,
    error::WorkerErrorV2,
    handlers::{email::send_email, webhook::send_webhook},
};

#[instrument(skip(pool, smtp_sender))]
pub async fn execute_job(
    pool: &PgPool,
    job: Job,
    worker_id: Uuid,
    smtp_sender: AsyncSmtpTransport<Tokio1Executor>,
    client: reqwest::Client,
) -> Result<(), WorkerErrorV2> {
    let job_id = job.id;

    let retries_exhausted = job.attempts == job.max_retries;

    let backoff_secs = retry_backoff_secs(job.attempts);

    let result = match job.job_type.as_ref() {
        "send_email" => send_email(smtp_sender, job.payload).await,
        "send_webhook" => send_webhook(client, job.payload).await,
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
        _ => Err(WorkerErrorV2::permanent("Invalid job type")),
    };

    match result {
        Ok(res) => {
            info!("Job completed");
            let moved_jobs =
                queries::move_job_record_to_completed(pool, job_id, worker_id, res).await?;
            if moved_jobs != 1 {
                error!(moved_jobs = moved_jobs, "Failed to mark job as completed");
            }
        }
        Err(err) => {
            error!(
                error = ?err,
                retry_backoff_time_in_secs = backoff_secs,
                "Failed to execute job"
            );
            let updated_rows =
                queries::store_job_error(pool, job_id, worker_id, err.to_string(), backoff_secs)
                    .await?;
            if updated_rows != 1 {
                error!(
                    updated_rows = updated_rows,
                    "Failed to update job error and retry time"
                );
            }

            if !err.is_retryable() || retries_exhausted {
                let moved_rows =
                    queries::move_job_record_to_failed(pool, job_id, worker_id).await?;
                if moved_rows == 1 {
                    info!("Moved the job to failed jobs");
                } else {
                    error!(moved_rows = moved_rows, "Failed to move job");
                }
            }
        }
    }

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
