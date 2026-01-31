use chrono::Utc;
use sqlx::{postgres::PgPool, query, query_as, types::JsonValue};
use uuid::Uuid;

use shared::db::models::{Job, JobStatus};

pub async fn register(pool: &PgPool, worker_id: Uuid, pid: i32) -> Result<(), sqlx::Error> {
    query("INSERT INTO workers (id, pid, started_at, last_heartbeat) VALUES ($1, $2, $3, $3);")
        .bind(worker_id)
        .bind(pid)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn heartbeat(pool: &PgPool, worker_id: Uuid) -> Result<(), sqlx::Error> {
    query("UPDATE workers SET last_heartbeat=$2 WHERE id=$1;")
        .bind(worker_id)
        .bind(Utc::now())
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn claim_job(pool: &PgPool, worker_id: Uuid) -> Result<Job, sqlx::Error> {
    query_as::<_, Job>(
        "UPDATE jobs
        SET
            status = $1,
            worker_id = $2,
            started_at = $3,
            attempts = attempts + 1
        WHERE id = (
            SELECT id FROM jobs
            WHERE status = $4
            AND attempts < max_retries
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
        )
        RETURNING *",
    )
    .bind(JobStatus::Running)
    .bind(worker_id)
    .bind(Utc::now())
    .bind(JobStatus::Pending)
    .fetch_one(pool)
    .await
}

pub async fn mark_job_as_completed(
    pool: &PgPool,
    job_id: Uuid,
    result: Option<JsonValue>,
) -> Result<(), sqlx::Error> {
    query(
        "UPDATE jobs
        SET
            status = $1,
            completed_at = $2,
            result = $3
        WHERE id = $4;",
    )
    .bind(JobStatus::Completed)
    .bind(Utc::now())
    .bind(result)
    .bind(job_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn store_job_error(
    pool: &PgPool,
    job_id: Uuid,
    error: String,
) -> Result<(), sqlx::Error> {
    query(
        "UPDATE jobs
        SET
            status = $1,
            error_message = $2
        WHERE id = $3;",
    )
    .bind(JobStatus::Pending)
    .bind(error)
    .bind(job_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_job_as_failed(pool: &PgPool, job_id: Uuid) -> Result<(), sqlx::Error> {
    query(
        "UPDATE jobs
        SET
            status = $1
        WHERE id = $2;",
    )
    .bind(JobStatus::Failed)
    .bind(job_id)
    .execute(pool)
    .await?;

    Ok(())
}
