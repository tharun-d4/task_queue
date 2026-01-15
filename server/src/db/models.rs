use chrono::{DateTime, Utc};
use sqlx::types::JsonValue;

use shared::db::models::JobStatus;

#[derive(Debug)]
pub struct NewJob {
    pub job_type: String,
    pub payload: JsonValue,
    pub status: JobStatus,
    pub priority: i8,
    pub max_retries: u8,
    pub created_at: DateTime<Utc>,
}
