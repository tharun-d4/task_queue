use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::QueryBuilder;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{error::ServerError, state::AppState};
use shared::db::{
    models::{CreateJob, Job, JobStatus},
    queries::{get_job_by_id, insert_job},
};

#[derive(Debug, Deserialize)]
pub struct JobPayload {
    job_type: String,
    payload: JsonValue,
    priority: Option<i16>,
    max_retries: Option<i16>,
    schedule_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobId {
    pub job_id: Uuid,
}

#[instrument(skip(state))]
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(job_payload): Json<JobPayload>,
) -> Result<(StatusCode, Json<JobId>), ServerError> {
    let job_id = insert_job(
        &state.pool,
        CreateJob {
            job_type: job_payload.job_type,
            payload: job_payload.payload,
            status: JobStatus::Pending,
            priority: job_payload.priority.unwrap_or(1),
            max_retries: job_payload.max_retries.unwrap_or(1),
            created_at: Utc::now(),
            run_at: job_payload.schedule_at.unwrap_or(Utc::now()),
        },
    )
    .await?;
    info!(%job_id, "Job Created");
    Ok((StatusCode::CREATED, Json(JobId { job_id })))
}

#[instrument(skip(state))]
pub async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, ServerError> {
    match get_job_by_id(&state.pool, id).await {
        Some(job) => Ok(Json(job)),
        None => Err(ServerError::NotFound("Job not found".to_string())),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    status: Option<JobStatus>,
    sort_by: Option<String>,
    order: Option<bool>,
    limit: i16,
    offset: i64,
}

#[derive(Debug, Serialize)]
pub struct ListJobsResponse {
    jobs: Vec<Job>,
    total: i64,
    limit: i16,
    offset: i64,
}

#[instrument(skip(state))]
pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListJobsQuery>,
) -> Result<Json<ListJobsResponse>, ServerError> {
    let mut jobs_query_builder = QueryBuilder::new("SELECT * FROM jobs");
    let mut job_count_query_builder = QueryBuilder::new("SELECT COUNT(id) FROM jobs");

    if let Some(status) = params.status {
        jobs_query_builder.push(" WHERE status = ");
        jobs_query_builder.push_bind(status);

        job_count_query_builder.push(" WHERE status = ");
        job_count_query_builder.push_bind(status);
    }

    if let Some(s) = params.sort_by {
        jobs_query_builder.push(match s.as_str() {
            "priority" => " ORDER BY priority",
            "started_at" => " ORDER BY started_at",
            "finished_at" => " ORDER BY finished_at",
            _ => " ORDER BY created_at",
        });

        jobs_query_builder.push(match params.order.unwrap_or(false) {
            true => " ASC",
            false => " DESC",
        });
    }
    jobs_query_builder.push(" LIMIT ");
    jobs_query_builder.push_bind(params.limit);

    jobs_query_builder.push(" OFFSET ");
    jobs_query_builder.push_bind(params.offset);

    info!(sql = jobs_query_builder.sql());

    let jobs = jobs_query_builder
        .build_query_as()
        .fetch_all(&state.pool)
        .await?;
    let job_count: i64 = job_count_query_builder
        .build_query_scalar()
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(ListJobsResponse {
        jobs,
        total: job_count,
        limit: params.limit,
        offset: params.offset,
    }))
}
