use shared::db::{models::JobStatus, queries::get_job_by_id};
use sqlx::postgres::PgPool;
use uuid::Uuid;
use worker::db::queries;

const JOB_LEASE_DURATION: u8 = 15;

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
    let job = queries::claim_job(&pool, worker_id, JOB_LEASE_DURATION)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(job.status, JobStatus::Running);
    assert_eq!(job.worker_id, Some(worker_id));
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn mark_job_as_completed(pool: PgPool) {
    let worker_id = Uuid::parse_str("019bfe1d-228e-7938-8678-3798f454c236").unwrap();
    let claimed_job = queries::claim_job(&pool, worker_id, JOB_LEASE_DURATION)
        .await
        .unwrap()
        .unwrap();
    queries::mark_job_as_completed(&pool, claimed_job.id, worker_id, None)
        .await
        .unwrap();

    let job = get_job_by_id(&pool, claimed_job.id).await.unwrap();
    assert_eq!(claimed_job.id, job.id);
    assert_eq!(job.status, JobStatus::Completed);
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("invalid_jobs"))
)]
async fn update_job_error_and_backoff_time(pool: PgPool) {
    let worker_id = Uuid::parse_str("019bfe1d-228e-7938-8678-3798f454c236").unwrap();

    let job = queries::claim_job(&pool, worker_id, JOB_LEASE_DURATION)
        .await
        .unwrap()
        .unwrap();
    queries::update_job_error_and_backoff_time(
        &pool,
        job.id,
        worker_id,
        "Invalid job".to_string(),
        10,
    )
    .await
    .unwrap();

    let job = get_job_by_id(&pool, job.id).await.unwrap();
    assert_eq!(job.error_message, Some("Invalid job".to_string()));
}
