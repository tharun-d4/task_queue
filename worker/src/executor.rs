use lettre::{transport::smtp::AsyncSmtpTransport, Tokio1Executor};
use sqlx::{postgres::PgPool, types::JsonValue};
use tracing::{error, info, instrument};

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
    smtp_sender: AsyncSmtpTransport<Tokio1Executor>,
    client: reqwest::Client,
) -> Result<(), WorkerError> {
    let job_id = job.id;

    let retry_limit_reached = job.max_retries == job.attempts.unwrap_or(0);

    let result = match job.job_type.as_ref() {
        "send_email" => send_email(smtp_sender, job).await,
        "send_webhook" => send_webhook(client, job.payload).await,
        _ => Err(WorkerError::InvalidJob),
    };

    match result {
        Ok(res) => {
            info!("Marking job as completed");
            queries::mark_job_as_completed(pool, job_id, res).await?;
        }
        Err(err) => {
            error!("Got error: {:?}", err);
            queries::store_job_error(pool, job_id, err.to_string()).await?;
            if retry_limit_reached {
                info!("Marking job as failed as the retry limit reached",);
                queries::mark_job_as_failed(pool, job_id).await?;
            }
        }
    }

    Ok(())
}

async fn send_email(
    smtp_sender: AsyncSmtpTransport<Tokio1Executor>,
    job: Job,
) -> Result<Option<JsonValue>, WorkerError> {
    let email_info: EmailInfo = serde_json::from_value(job.payload).unwrap();
    info!("Sending an email: {:?}", email_info);
    email::send_email(smtp_sender, email_info).await?;
    Ok(None)
}
