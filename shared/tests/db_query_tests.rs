use chrono::Utc;
use shared::db::{
    models::{CreateJob, JobStatus, RunMode},
    queries,
};
use sqlx::{PgPool, types::JsonValue};
use uuid::Uuid;

#[sqlx::test(migrations = "../migrations")]
async fn insert_job_returns_job_id(pool: PgPool) -> Result<(), sqlx::Error> {
    let job_id = queries::insert_job(
        &pool,
        CreateJob {
            job_type: "new_job".to_string(),
            run_mode: RunMode::Immediate,
            payload: JsonValue::String("A new job".to_string()),
            cron_expression: None,
            status: JobStatus::Pending,
            priority: 1,
            max_retries: 5,
            created_at: Utc::now(),
            run_at: Utc::now(),
            parent_job_id: None,
        },
    )
    .await?;
    assert_eq!(job_id.get_version_num(), 7);
    Ok(())
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn get_job_by_id_returns_job(pool: PgPool) {
    let job_id = "019bfadc-28bb-781d-9d22-acf23fe50117"
        .parse::<Uuid>()
        .unwrap();
    let job = queries::get_job_by_id(&pool, job_id).await;

    assert!(job.is_some());
    assert_eq!(job_id, job.unwrap().id);
}
