use chrono::{TimeDelta, Utc};
use shared::db::models::{Job, JobStatus};
use sqlx::{postgres::PgPool, query, query_as, types::JsonValue};
use uuid::Uuid;

use crate::error::WorkerError;

pub async fn register(pool: &PgPool, worker_id: Uuid, pid: i32) -> Result<(), WorkerError> {
    query(
        "
        INSERT INTO workers
        (id, pid, started_at, last_heartbeat)
        VALUES ($1, $2, $3, $3);
        ",
    )
    .bind(worker_id)
    .bind(pid)
    .bind(Utc::now())
    .execute(pool)
    .await
    .map_err(|e| WorkerError::temporary("Failed to register worker").set_source(e))?;

    Ok(())
}

pub async fn update_heartbeat(pool: &PgPool, worker_id: Uuid) -> Result<u64, WorkerError> {
    let rows_affected = query(
        "
        UPDATE workers
        SET last_heartbeat=$2
        WHERE id=$1;
        ",
    )
    .bind(worker_id)
    .bind(Utc::now())
    .execute(pool)
    .await
    .map_err(|e| WorkerError::temporary("Failed to update worker heartbeat").set_source(e))?
    .rows_affected();

    Ok(rows_affected)
}

pub async fn claim_job(
    pool: &PgPool,
    worker_id: Uuid,
    lease_duration: u8,
) -> Result<Option<Job>, WorkerError> {
    query_as::<_, Job>(
        "
        WITH pending_job AS (
            SELECT id
            FROM jobs
            WHERE status = $1
                AND attempts < max_retries
                AND run_at < NOW()
            ORDER BY priority DESC, created_at ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE jobs
        SET
            status = $2,
            worker_id = $3,
            started_at = $4,
            lease_expires_at = $5,
            attempts = attempts + 1
        FROM pending_job
        WHERE jobs.id = pending_job.id
        RETURNING jobs.*;
        ",
    )
    .bind(JobStatus::Pending)
    .bind(JobStatus::Running)
    .bind(worker_id)
    .bind(Utc::now())
    .bind(Utc::now() + TimeDelta::seconds(lease_duration as i64))
    .fetch_optional(pool)
    .await
    .map_err(|e| WorkerError::temporary("Failed to claim a job").set_source(e))
}

pub async fn mark_job_as_completed(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: Uuid,
    result: Option<JsonValue>,
) -> Result<u64, WorkerError> {
    let rows_affected = query(
        "
        UPDATE jobs
        SET
            status = $1,
            finished_at = $2,
            result = $3
        WHERE id = $4
            AND worker_id = $5
            AND status = $6;
        ",
    )
    .bind(JobStatus::Completed)
    .bind(Utc::now())
    .bind(result)
    .bind(job_id)
    .bind(worker_id)
    .bind(JobStatus::Running)
    .execute(pool)
    .await
    .map_err(|e| WorkerError::temporary("Failed to mark the job as completed").set_source(e))?
    .rows_affected();

    Ok(rows_affected)
}

pub async fn update_job_error_and_backoff_time(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: Uuid,
    error: String,
    backoff_secs: i16,
) -> Result<u64, WorkerError> {
    let rows_affected = query(
        "
        UPDATE jobs
        SET
            status = $1,
            error_message = $2,
            run_at = NOW() + ($3 * INTERVAL '1 SECONDS')
        WHERE id = $4
        AND worker_id = $5;
        ",
    )
    .bind(JobStatus::Pending)
    .bind(error)
    .bind(backoff_secs)
    .bind(job_id)
    .bind(worker_id)
    .execute(pool)
    .await
    .map_err(|e| WorkerError::temporary("Failed to update the job error").set_source(e))?
    .rows_affected();

    Ok(rows_affected)
}

pub async fn mark_job_as_failed(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: Uuid,
    error: String,
) -> Result<u64, WorkerError> {
    let rows_affected = query(
        "
        UPDATE jobs
        SET
            status = $1,
            finished_at = $2,
            error_message = $3
        WHERE id = $4
            AND worker_id = $5
            AND status = $6;
        ",
    )
    .bind(JobStatus::Failed)
    .bind(Utc::now())
    .bind(error)
    .bind(job_id)
    .bind(worker_id)
    .bind(JobStatus::Running)
    .execute(pool)
    .await
    .map_err(|e| WorkerError::temporary("Failed to mark the job as failed").set_source(e))?
    .rows_affected();

    Ok(rows_affected)
}
pub async fn update_worker_shutdown_time(
    pool: &PgPool,
    worker_id: Uuid,
) -> Result<(), WorkerError> {
    query(
        "
        UPDATE workers
        SET shutdown_at = NOW()
        WHERE id = $1;
        ",
    )
    .bind(worker_id)
    .execute(pool)
    .await
    .map_err(|e| WorkerError::temporary("Failed to update worker shutdown time").set_source(e))?;

    Ok(())
}
