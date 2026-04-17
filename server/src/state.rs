use std::sync::Arc;

use prometheus_client::registry::Registry;
use sqlx::postgres::PgPool;

use crate::prometheus::Metrics;
//use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub registry: Arc<Registry>,
    pub metrics: Arc<Metrics>,
}

impl AppState {
    pub fn new(pool: PgPool, registry: Registry, metrics: Metrics) -> Self {
        AppState {
            pool,
            registry: Arc::new(registry),
            metrics: Arc::new(metrics),
        }
    }
}
