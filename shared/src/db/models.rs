use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, types::JsonValue};
use uuid::Uuid;

#[derive(Debug, sqlx::Type, Serialize)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: JsonValue,
    pub status: JobStatus,
    pub priority: i16,
    pub max_retries: i16,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub worker_id: Option<Uuid>,
    pub attempts: Option<i16>,
    pub error_message: Option<String>,
    pub result: Option<JsonValue>,
}

#[derive(Debug)]
pub struct NewJob {
    pub job_type: String,
    pub payload: JsonValue,
    pub status: JobStatus,
    pub priority: i16,
    pub max_retries: i16,
    pub created_at: DateTime<Utc>,
}
