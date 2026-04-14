use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct JobStats {
    pub pending: i64,
    pub running: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
}

#[derive(Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct JobStatsByJobType {
    pub job_type: String,
    pub pending: i64,
    pub running: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
}
