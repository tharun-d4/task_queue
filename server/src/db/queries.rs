use chrono::Utc;
use shared::db::models::JobStatus;
use sqlx::{postgres::PgPool, query, query_as};
use uuid::Uuid;

use crate::db::models::{JobStats, JobStatsByJobType};

pub async fn recover_unfinished_lease_expired_jobs(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let jobs_recovered = query!(
        "
        UPDATE jobs
        SET status = 'pending',
        error_message = 'lease expired or worker crashed'
        WHERE status = 'running'
        AND lease_expires_at < NOW()
        ",
    )
    .execute(pool)
    .await?
    .rows_affected();

    Ok(jobs_recovered)
}

pub async fn mark_retry_exhausted_jobs_as_failed(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let moved = query!(
        "
        UPDATE jobs
        SET status = 'failed',
            finished_at = NOW()
        WHERE status = 'pending'
            AND attempts >= max_retries
        ",
    )
    .execute(pool)
    .await?
    .rows_affected();

    Ok(moved)
}

pub async fn get_job_stats(pool: &PgPool) -> Result<JobStats, sqlx::Error> {
    query_as(
        "
        SELECT
            COUNT(id) FILTER(WHERE status='pending') as pending,
            COUNT(id) FILTER(WHERE status='running') as running,
            COUNT(id) FILTER(WHERE status='completed') as completed,
            COUNT(id) FILTER(WHERE status='failed') as failed
        FROM jobs;
        ",
    )
    .fetch_one(pool)
    .await
}

pub async fn get_job_stats_by_job_type(
    pool: &PgPool,
) -> Result<Vec<JobStatsByJobType>, sqlx::Error> {
    query_as(
        "
        SELECT
            job_type,
            COUNT(id) FILTER(WHERE status='pending') as pending,
            COUNT(id) FILTER(WHERE status='running') as running,
            COUNT(id) FILTER(WHERE status='completed') as completed,
            COUNT(id) FILTER(WHERE status='failed') as failed
        FROM jobs
        GROUP BY job_type;
        ",
    )
    .fetch_all(pool)
    .await
}

pub async fn cancel_job(pool: &PgPool, job_id: Uuid) -> Result<u64, sqlx::Error> {
    let rows_affected = query(
        "
            UPDATE jobs
            SET status = $1,
            finished_at = $2
            WHERE id = $3
            AND status = $4
            ",
    )
    .bind(JobStatus::Cancelled)
    .bind(Utc::now())
    .bind(job_id)
    .bind(JobStatus::Pending)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(rows_affected)
}
