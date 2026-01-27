use sqlx::postgres::PgPool;
use uuid::Uuid;

use worker::db::queries;

#[sqlx::test(migrations = "../migrations")]
async fn register_worker_test(pool: PgPool) {
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
async fn claim_job_test(pool: PgPool) {
    let worker_id = Uuid::parse_str("019bfe1d-228e-7938-8678-3798f454c236").unwrap();
    queries::claim_job(&pool, worker_id).await.unwrap();
}
