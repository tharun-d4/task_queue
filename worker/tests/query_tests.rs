use sqlx::postgres::PgPool;
use uuid::Uuid;

use shared::db::{models::JobStatus, queries::get_job_by_id};
use worker::db::queries;

#[sqlx::test(migrations = "../migrations")]
async fn register_worker(pool: PgPool) {
    let worker_id = Uuid::now_v7();
    let pid = std::process::id();

    queries::register(&pool, worker_id, pid as i32)
        .await
        .unwrap();
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs", "workers"))
)]
async fn claim_job_returns_job(pool: PgPool) {
    let worker_id = Uuid::parse_str("019bfe1d-228e-7938-8678-3798f454c236").unwrap();
    let job = queries::claim_job(&pool, worker_id).await.unwrap();

    assert_eq!(job.status, JobStatus::Running);
    assert_eq!(job.worker_id, Some(worker_id));
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs", "workers"))
)]
async fn mark_job_as_completed(pool: PgPool) {
    let job_id = Uuid::parse_str("019bfadc-28bb-781d-9d22-acf23fe50117").unwrap();

    queries::mark_job_as_completed(&pool, job_id, None)
        .await
        .unwrap();

    let job = get_job_by_id(&pool, job_id).await.unwrap();
    assert_eq!(job.id, job_id);
    assert_eq!(job.status, JobStatus::Completed);
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs", "workers"))
)]
async fn mark_job_as_failed(pool: PgPool) {
    let job_id = Uuid::parse_str("019bfdd5-cc70-7f37-a02a-1ec5849f25df").unwrap();

    queries::mark_job_as_failed(&pool, job_id).await.unwrap();

    let job = get_job_by_id(&pool, job_id).await.unwrap();
    assert_eq!(job.id, job_id);
    assert_eq!(job.status, JobStatus::Failed);
}
