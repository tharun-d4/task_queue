use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{info, instrument};

use crate::{error::ServerError, state::AppState};
use shared::db::{
    models::{JobStatus, NewJob},
    queries::insert_job,
};

#[derive(Debug, Deserialize)]
pub struct JobPayload {
    job_type: String,
    payload: JsonValue,
    priority: Option<i8>,
    max_retries: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct JobId {
    job_id: uuid::Uuid,
}

#[instrument(skip(state))]
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(req_payload): Json<JobPayload>,
) -> Result<(StatusCode, Json<JobId>), ServerError> {
    let state = state.clone();

    let job_id = insert_job(
        &state.pool,
        NewJob {
            job_type: req_payload.job_type,
            payload: req_payload.payload,
            status: JobStatus::Pending,
            priority: match req_payload.priority {
                Some(val) => val,
                None => 1,
            },
            max_retries: match req_payload.max_retries {
                Some(val) => val,
                None => 1,
            },
            created_at: Utc::now(),
        },
    )
    .await?;
    info!(%job_id, "Job Created");
    Ok((StatusCode::CREATED, Json(JobId { job_id })))
}
