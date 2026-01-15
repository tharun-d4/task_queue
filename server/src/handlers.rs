use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::{
    db::{models, queries},
    error::ServerError,
    state::AppState,
};
use shared::db::models::JobStatus;

#[derive(Debug, Deserialize)]
pub struct JobPayload {
    job_type: String,
    payload: JsonValue,
    priority: Option<i8>,
    max_retries: Option<u8>,
}

pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(req_payload): Json<JobPayload>,
) -> Result<impl IntoResponse, ServerError> {
    let state = state.clone();

    let job_id = queries::insert_job(
        &state.pool,
        models::NewJob {
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
    Ok(job_id.to_string())
}
