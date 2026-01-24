use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Worker {
    pub id: Uuid,
    pub pid: i32,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub shutdown_at: Option<DateTime<Utc>>,
}
