use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::JsonValue};
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: JsonValue,
    pub status: JobStatus,
    pub priority: i16,
    pub max_retries: i16,
    pub created_at: DateTime<Utc>,
    pub run_at: DateTime<Utc>,
    pub worker_id: Option<Uuid>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub attempts: i16,
    pub error_message: Option<String>,
    pub result: Option<JsonValue>,
}

#[derive(Debug)]
pub struct CreateJob {
    pub job_type: String,
    pub payload: JsonValue,
    pub status: JobStatus,
    pub priority: i16,
    pub max_retries: i16,
    pub created_at: DateTime<Utc>,
    pub run_at: DateTime<Utc>,
}
