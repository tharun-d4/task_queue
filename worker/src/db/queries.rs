use chrono::Utc;
use sqlx::{postgres::PgPool, query};
use uuid::Uuid;

pub async fn register(pool: &PgPool, id: Uuid, pid: i32) -> Result<(), sqlx::Error> {
    query("INSERT INTO workers (id, pid, started_at, last_heartbeat) VALUES ($1, $2, $3, $3);")
        .bind(id)
        .bind(pid)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    Ok(())
}
