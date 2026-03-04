use chrono::{TimeDelta, Utc};
use sqlx::{postgres::PgPool, query, query_as, types::JsonValue};
use uuid::Uuid;

use crate::error::WorkerErrorV2;
use shared::db::models::{Job, JobStatus};

pub async fn register(pool: &PgPool, worker_id: Uuid, pid: i32) -> Result<(), WorkerErrorV2> {
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
    .map_err(|e| WorkerErrorV2::temporary("Failed to register worker").set_source(e))?;

    Ok(())
}

pub async fn update_heartbeat(pool: &PgPool, worker_id: Uuid) -> Result<u64, sqlx::Error> {
    let updated_rows = query("UPDATE workers SET last_heartbeat=$2 WHERE id=$1;")
        .bind(worker_id)
        .bind(Utc::now())
        .execute(pool)
        .await?
        .rows_affected();

    Ok(updated_rows)
}

pub async fn claim_job(
    pool: &PgPool,
    worker_id: Uuid,
    lease_duration: u8,
) -> Result<Option<Job>, sqlx::Error> {
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
}

//pub async fn mark_job_as_completed(
//    pool: &PgPool,
//    job_id: Uuid,
//    worker_id: Uuid,
//    result: Option<JsonValue>,
//) -> Result<u64, sqlx::Error> {
//    let updated_rows = query(
//        "UPDATE jobs
//        SET
//            status = $1,
//            completed_at = $2,
//            result = $3
//        WHERE id = $4
//        AND worker_id = $5
//        AND status = $6;",
//    )
//    .bind(JobStatus::Completed)
//    .bind(Utc::now())
//    .bind(result)
//    .bind(job_id)
//    .bind(worker_id)
//    .bind(JobStatus::Running)
//    .execute(pool)
//    .await?
//    .rows_affected();
//
//    Ok(updated_rows)
//}

pub async fn move_job_record_to_completed(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: Uuid,
    result: Option<JsonValue>,
) -> Result<u64, sqlx::Error> {
    let rows_affected = query(
        "
        WITH completed AS (
            DELETE FROM jobs
            WHERE id = $1
            AND worker_id = $2
            AND status = $3
            RETURNING
                id,
                job_type,
                payload,
                priority,
                max_retries,
                created_at,
                run_at,
                started_at,
                worker_id,
                lease_expires_at,
                attempts,
                error_message
        )
        INSERT INTO completed_jobs
        (
            id,
            job_type,
            payload,
            priority,
            max_retries,
            created_at,
            run_at,
            started_at,
            worker_id,
            lease_expires_at,
            attempts,
            error_message,
            completed_at,
            result
        )
        SELECT
            *,
            $4,
            $5
        FROM completed;
        ",
    )
    .bind(job_id)
    .bind(worker_id)
    .bind(JobStatus::Running)
    .bind(Utc::now())
    .bind(result)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(rows_affected)
}

pub async fn move_job_record_to_failed(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let moved_rows = query(
        "
        WITH deleted_job AS (
            DELETE FROM jobs
            WHERE id = $1
                AND worker_id = $2
            RETURNING
                id,
                job_type,
                payload,
                priority,
                max_retries,
                created_at,
                started_at,
                NOW(),
                worker_id,
                attempts,
                error_message,
                result,
                lease_expires_at
        )
        INSERT INTO failed_jobs
        SELECT * FROM deleted_job;
        ",
    )
    .bind(job_id)
    .bind(worker_id)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(moved_rows)
}

pub async fn store_job_error(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: Uuid,
    error: String,
    backoff_secs: i16,
) -> Result<u64, sqlx::Error> {
    let updated_rows = query(
        "UPDATE jobs
        SET
            status = $1,
            error_message = $2,
            run_at = NOW() + ($3 * INTERVAL '1 SECONDS')
        WHERE id = $4
        AND worker_id = $5;",
    )
    .bind(JobStatus::Pending)
    .bind(error)
    .bind(backoff_secs)
    .bind(job_id)
    .bind(worker_id)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(updated_rows)
}

pub async fn update_worker_shutdown_time(
    pool: &PgPool,
    worker_id: Uuid,
) -> Result<(), WorkerErrorV2> {
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
    .map_err(|e| WorkerErrorV2::temporary("Failed to update worker shutdown time").set_source(e))?;

    Ok(())
}
