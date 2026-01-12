use chrono::Utc;
use sqlx::{PgPool, types::JsonValue};
use uuid::Uuid;

use api_server::db::{models::*, queries};

#[sqlx::test(migrations = "../migrations")]
async fn test_insert_task_returns_task_id(pool: PgPool) -> Result<(), sqlx::Error> {
    let task_id = queries::insert_task(
        &pool,
        NewTask {
            id: Uuid::now_v7(),
            task_type: "new_task".to_string(),
            payload: JsonValue::String("A new task".to_string()),
            status: TaskStatus::Pending,
            priority: 1,
            max_retries: 5,
            created_at: Utc::now(),
        },
    )
    .await?;
    println!("task_id: {:?}", task_id);
    Ok(())
}
