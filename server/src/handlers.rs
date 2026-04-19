use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use prometheus_client::encoding::text::encode;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use shared::db::{
    models::{CreateJob, Job, JobStatus, RunMode},
    queries as shared_queries,
};
use sqlx::QueryBuilder;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{
    db::{
        models::{JobStats, JobStatsByJobType},
        queries,
    },
    error::ServerError,
    prometheus::JobType,
    state::AppState,
    utils::cron_parsed_to_time,
};

pub async fn handler_404() -> Result<(), ServerError> {
    Err(ServerError::NotFound(
        "Careful! You are calling an API that doesn't exist".to_string(),
    ))
}

pub async fn get_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buffer = String::new();

    encode(&mut buffer, &state.registry).unwrap();

    Response::builder()
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(Body::from(buffer))
        .unwrap()
}

#[derive(Debug, Deserialize)]
pub struct JobPayload {
    job_type: String,
    payload: JsonValue,
    priority: Option<i16>,
    max_retries: Option<i16>,
    schedule_at: Option<DateTime<Utc>>,
    cron_expression: Option<String>,
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
    let mut run_at = Utc::now();
    let mut run_mode = RunMode::Immediate;

    if let Some(time) = job_payload.schedule_at {
        run_at = time;
        run_mode = RunMode::Scheduled;
    }

    if let Some(ref expr) = job_payload.cron_expression {
        run_at = cron_parsed_to_time(expr, true)?;
        run_mode = RunMode::Recurring;
    }

    let job_id = shared_queries::insert_job(
        &state.pool,
        CreateJob {
            run_mode: run_mode,
            job_type: job_payload.job_type.clone(),
            payload: job_payload.payload,
            cron_expression: job_payload.cron_expression,
            status: JobStatus::Pending,
            priority: job_payload.priority.unwrap_or(1),
            max_retries: job_payload.max_retries.unwrap_or(1),
            created_at: Utc::now(),
            run_at: run_at,
            parent_job_id: None,
        },
    )
    .await?;
    info!(%job_id, "Job Created");

    state
        .metrics
        .jobs_submitted
        .get_or_create(&JobType {
            job_type: job_payload.job_type,
        })
        .inc();
    Ok((StatusCode::CREATED, Json(JobId { job_id })))
}

#[instrument(skip(state))]
pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ServerError> {
    let mut txn = state.pool.begin().await?;

    let job_status = queries::get_job_status(&mut *txn, id).await?;

    match job_status {
        JobStatus::Pending => {
            let rows_affected = queries::cancel_job(&mut *txn, id).await?;

            if rows_affected != 1 {
                let error = String::from("Too late! The job started executing.");
                error!(job_id = ?id, error = error);

                Err(ServerError::BadRequest(error.into()))
            } else {
                info!(job_id = ?id, "Job is cancelled");
                txn.commit().await?;

                Ok(StatusCode::NO_CONTENT)
            }
        }
        others => Err(ServerError::BadRequest(format!(
            "Cannot cancel the job. Current status: {:?}",
            others
        ))),
    }
}

#[instrument(skip(state))]
pub async fn get_job_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, ServerError> {
    match shared_queries::get_job_by_id(&state.pool, id).await {
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JobStatsResponse {
    pub overall: JobStats,
    pub by_job_type: Vec<JobStatsByJobType>,
}

pub async fn job_stats(State(state): State<Arc<AppState>>) -> Result<Json<JobStats>, ServerError> {
    let overall = queries::get_job_stats(&state.pool).await?;
    info!(overall = ?overall);
    Ok(Json(overall))
}

pub async fn detailed_job_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<JobStatsResponse>, ServerError> {
    let overall = queries::get_job_stats(&state.pool).await?;
    let by_job_type = queries::get_job_stats_by_job_type(&state.pool).await?;
    info!(overall = ?overall, by_job_type = ?by_job_type);

    Ok(Json(JobStatsResponse {
        overall,
        by_job_type,
    }))
}
