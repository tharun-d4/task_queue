use sqlx::{Executor, Postgres, QueryBuilder, postgres::PgPool, query_as, query_scalar};
use uuid::Uuid;

use crate::db::models::{CreateJob, Job};

pub async fn insert_job(pool: &PgPool, job: CreateJob) -> Result<Uuid, sqlx::Error> {
    let job_id = query_scalar(
        "INSERT INTO jobs
        (id, job_type, payload, status, priority, max_retries, created_at, run_at, run_mode, cron_expression)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id",
    )
    .bind(Uuid::now_v7())
    .bind(job.job_type)
    .bind(job.payload)
    .bind(job.status)
    .bind(job.priority)
    .bind(job.max_retries)
    .bind(job.created_at)
    .bind(job.run_at)
    .bind(job.run_mode)
    .bind(job.cron_expression)
    .fetch_one(pool)
    .await?;

    Ok(job_id)
}

pub async fn get_job_by_id(pool: &PgPool, id: Uuid) -> Option<Job> {
    query_as::<_, Job>("SELECT * FROM jobs WHERE id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn bulk_insert_jobs<'c, E>(executor: E, jobs: Vec<CreateJob>) -> Result<u64, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let mut query_builder = QueryBuilder::new(
        "
        INSERT INTO jobs
        (id, job_type, payload, status, priority, max_retries, created_at, run_mode, run_at, cron_expression, parent_job_id)
        ",
    );

    query_builder.push_values(jobs.into_iter(), |mut b, job| {
        b.push_bind(Uuid::now_v7())
            .push_bind(job.job_type)
            .push_bind(job.payload)
            .push_bind(job.status)
            .push_bind(job.priority)
            .push_bind(job.max_retries)
            .push_bind(job.created_at)
            .push_bind(job.run_mode)
            .push_bind(job.run_at)
            .push_bind(job.cron_expression)
            .push_bind(job.parent_job_id);
    });

    let inserted = query_builder
        .build()
        .execute(executor)
        .await?
        .rows_affected();

    Ok(inserted)
}
