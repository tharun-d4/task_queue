use chrono::{TimeDelta, Utc};
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

pub async fn update_heartbeat(pool: &PgPool, worker_id: Uuid) -> Result<(), sqlx::Error> {
    query("UPDATE workers SET last_heartbeat=$2 WHERE id=$1;")
        .bind(worker_id)
        .bind(Utc::now())
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn claim_job(
    pool: &PgPool,
    worker_id: Uuid,
    lease_duration: u8,
) -> Result<Job, sqlx::Error> {
    query_as::<_, Job>(
        "UPDATE jobs
        SET
            status = $1,
            worker_id = $2,
            started_at = $3,
            lease_expires_at = $4,
            attempts = attempts + 1
        WHERE id = (
            SELECT id FROM jobs
            WHERE status = $5
            AND attempts < max_retries
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
        )
        RETURNING *",
    )
    .bind(JobStatus::Running)
    .bind(worker_id)
    .bind(Utc::now())
    .bind(Utc::now() + TimeDelta::seconds(lease_duration as i64))
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

pub async fn move_to_failed_jobs(pool: &PgPool, job_id: Uuid) -> Result<(), sqlx::Error> {
    let job = query_as::<_, Job>("SELECT * FROM jobs WHERE id=$1")
        .bind(job_id)
        .fetch_one(pool)
        .await?;

    query(
        "INSERT INTO failed_jobs
        (
            id,
            job_type,
            payload,
            priority,
            max_retries,
            created_at,
            started_at,
            failed_at,
            worker_id,
            attempts,
            error_message,
            result
        )
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);
        ",
    )
    .bind(job.id)
    .bind(job.job_type)
    .bind(job.payload)
    .bind(job.priority)
    .bind(job.max_retries)
    .bind(job.created_at)
    .bind(job.started_at)
    .bind(Utc::now())
    .bind(job.worker_id)
    .bind(job.attempts)
    .bind(job.error_message)
    .bind(job.result)
    .execute(pool)
    .await?;

    delete_job(&pool, job_id).await?;

    Ok(())
}

pub async fn delete_job(pool: &PgPool, job_id: Uuid) -> Result<(), sqlx::Error> {
    query(
        "DELETE FROM jobs
        WHERE id = $1;",
    )
    .bind(job_id)
    .execute(pool)
    .await?;

    Ok(())
}
