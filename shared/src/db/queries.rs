use sqlx::{postgres::PgPool, query_scalar};
use uuid::Uuid;

use crate::db::models::NewJob;

pub async fn insert_job(pool: &PgPool, job: NewJob) -> Result<Uuid, sqlx::Error> {
    let job_id = query_scalar(
        "INSERT INTO jobs
        VALUES (
            $1, $2, $3, $4, $5, $6, $7
        )
        RETURNING id",
    )
    .bind(Uuid::now_v7())
    .bind(job.job_type)
    .bind(job.payload)
    .bind(job.status)
    .bind(job.priority as i16)
    .bind(job.max_retries as i16)
    .bind(job.created_at)
    .fetch_one(pool)
    .await?;

    Ok(job_id)
}
