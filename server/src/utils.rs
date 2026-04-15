use std::str::FromStr;

use chrono::{DateTime, Utc};
use croner::Cron;
use tracing::error;

use crate::error::ServerError;

pub fn cron_parsed_to_time(expr: &str, inclusive: bool) -> Result<DateTime<Utc>, ServerError> {
    let cron = Cron::from_str(expr).map_err(|err| {
        error!(error = ?err);
        ServerError::BadRequest("Invalid cron expression for a recurring job.".into())
    })?;

    cron.find_next_occurrence(&Utc::now(), inclusive)
        .map_err(|e| ServerError::Internal(e.to_string()))
}
