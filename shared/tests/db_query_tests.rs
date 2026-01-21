use chrono::Utc;
use sqlx::{PgPool, types::JsonValue};

use shared::db::{
    models::{JobStatus, NewJob},
    queries,
};

#[sqlx::test(migrations = "../migrations")]
async fn test_insert_job_returns_job_id(pool: PgPool) -> Result<(), sqlx::Error> {
    let job_id = queries::insert_job(
        &pool,
        NewJob {
            job_type: "new_job".to_string(),
            payload: JsonValue::String("A new job".to_string()),
            status: JobStatus::Pending,
            priority: 1,
            max_retries: 5,
            created_at: Utc::now(),
        },
    )
    .await?;
    Ok(())
}
