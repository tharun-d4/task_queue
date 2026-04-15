use chrono::Utc;
use shared::db::models::{JobStatus, RecurringJob, RunMode};
use sqlx::{Executor, Postgres, postgres::PgPool, query, query_as, query_scalar};
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

pub async fn get_recurring_jobs_to_reschedule<'c, E>(
    executor: E,
) -> Result<Vec<RecurringJob>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let rows: Vec<RecurringJob> = query_as(
        "
            SELECT
                id,
                job_type,
                payload,
                cron_expression,
                priority,
                max_retries,
                parent_job_id
            FROM jobs
            WHERE
                status = $1
                AND run_mode = $2
                AND cron_expression IS NOT NULL
                AND rescheduled = FALSE
        ",
    )
    .bind(JobStatus::Completed)
    .bind(RunMode::Recurring)
    .fetch_all(executor)
    .await?;

    Ok(rows)
}

pub async fn mark_recurring_jobs_as_rescheduled<'c, E>(
    executor: E,
    job_ids: &[Uuid],
) -> Result<u64, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let updated = query(
        "
            UPDATE jobs
            SET rescheduled = TRUE
            WHERE id = Any($1)
        ",
    )
    .bind(job_ids)
    .execute(executor)
    .await?
    .rows_affected();

    Ok(updated)
}

pub async fn get_job_stats(pool: &PgPool) -> Result<JobStats, sqlx::Error> {
    query_as(
        "
        SELECT
            COUNT(id) FILTER(WHERE status='pending') as pending,
            COUNT(id) FILTER(WHERE status='running') as running,
            COUNT(id) FILTER(WHERE status='completed') as completed,
            COUNT(id) FILTER(WHERE status='failed') as failed,
            COUNT(id) FILTER(WHERE status='failed') as cancelled
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
            COUNT(id) FILTER(WHERE status='failed') as failed,
            COUNT(id) FILTER(WHERE status='failed') as cancelled
        FROM jobs
        GROUP BY job_type;
        ",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_job_status<'c, E>(executor: E, job_id: Uuid) -> Result<JobStatus, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    query_scalar(
        "
            SELECT status
            FROM jobs
            WHERE id = $1;
            ",
    )
    .bind(job_id)
    .fetch_one(executor)
    .await
}

pub async fn cancel_job<'c, E>(executor: E, job_id: Uuid) -> Result<u64, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
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
    .execute(executor)
    .await?
    .rows_affected();

    Ok(rows_affected)
}
