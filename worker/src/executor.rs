use sqlx::{postgres::PgPool, types::JsonValue};
use tracing::{error, info, instrument};

use shared::db::models::Job;

use crate::db::queries;

#[instrument(skip(pool))]
pub async fn execute_job(pool: &PgPool, job: Job) {
    let job_id = job.id;

    let retry_limit_reached = job.max_retries == job.attempts.unwrap_or(0);

    let result = match job.job_type.as_ref() {
        "send_email" => send_email(job).await,
        _ => Err("Unknown Job Type Found".to_string()),
    };

    match result {
        Ok(res) => {
            queries::mark_job_as_completed(pool, job_id, res)
                .await
                .unwrap();
        }
        Err(err) => {
            error!("Got error: {:?}", err);
            queries::store_job_error(pool, job_id, err).await.unwrap();
            if retry_limit_reached {
                queries::mark_job_as_failed(pool, job_id).await.unwrap();
            }
        }
    }
}

async fn send_email(_job: Job) -> Result<Option<JsonValue>, String> {
    info!("Got a send_email job to do. Performing it ...");
    Ok(None)
}
