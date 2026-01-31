use sqlx::types::JsonValue;
use tracing::info;

use shared::db::models::Job;

pub async fn execute_job(job: Job) -> Result<Option<JsonValue>, String> {
    match job.job_type.as_ref() {
        "send_email" => {
            info!("Got a send_email job to do. Performing it ...");
            Ok(None)
        }
        _ => Err("Unknown Job Type Found".to_string()),
    }
}
