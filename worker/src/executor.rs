use lettre::{Tokio1Executor, transport::smtp::AsyncSmtpTransport};
use sqlx::{postgres::PgPool, types::JsonValue};
use tracing::{error, info, instrument};
use uuid::Uuid;

use shared::db::models::Job;

use crate::{
    db::queries,
    error::WorkerError,
    handlers::{email, models::EmailInfo, webhook::send_webhook},
};

#[instrument(skip(pool, smtp_sender))]
pub async fn execute_job(
    pool: &PgPool,
    job: Job,
    worker_id: Uuid,
    smtp_sender: AsyncSmtpTransport<Tokio1Executor>,
    client: reqwest::Client,
) -> Result<(), WorkerError> {
    let job_id = job.id;

    let retries_exhausted = job.attempts == job.max_retries;

    let backoff_secs = retry_backoff_secs(job.attempts);

    let result = match job.job_type.as_ref() {
        "send_email" => send_email(smtp_sender, job).await,
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
        _ => Err(WorkerError::InvalidJob),
    };

    match result {
        Ok(res) => {
            let updated_rows =
                queries::move_job_record_to_completed(pool, job_id, worker_id, res).await?;
            if updated_rows == 1 {
                info!("Job marked as completed");
            } else {
                error!(
                    updated_rows = updated_rows,
                    "Failed to mark job as completed"
                );
            }
        }
        Err(err) => {
            error!(
                error = ?err,
                retry_backoff_time_in_secs = backoff_secs
            );
            let updated_rows =
                queries::store_job_error(pool, job_id, worker_id, err.to_string(), backoff_secs)
                    .await?;
            if updated_rows == 1 {
                info!("Updated job error and retry backoff time");
            } else {
                error!(
                    updated_rows = updated_rows,
                    "Failed to update job error and retry time"
                );
            }
            if retries_exhausted {
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

async fn send_email(
    smtp_sender: AsyncSmtpTransport<Tokio1Executor>,
    job: Job,
) -> Result<Option<JsonValue>, WorkerError> {
    let email_info: EmailInfo = serde_json::from_value(job.payload)
        .map_err(|e| WorkerError::Email(format!("email payload json error: {:?}", e)))?;

    info!("Sending an email: {:?}", email_info);
    email::send_email(smtp_sender, email_info).await?;
    Ok(None)
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
