use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use shared::db::{
    models::{CreateJob, JobStatus, RunMode},
    queries::bulk_insert_jobs,
};
use sqlx::PgPool;
use tracing::{error, info};

use crate::{
    db::queries::{self, mark_recurring_jobs_as_rescheduled},
    error::ServerError,
    prometheus::JobType,
    state::AppState,
    utils::cron_parsed_to_time,
};

pub async fn reschedule_recurring_jobs(
    pool: &PgPool,
    state: &Arc<AppState>,
) -> Result<(), ServerError> {
    let mut txn = pool.begin().await?;
    let jobs = queries::get_recurring_jobs_to_reschedule(&mut *txn).await?;

    if jobs.is_empty() {
        return Err(ServerError::NotFound(
            "No recurring jobs found to be rescheduled".into(),
        ));
    }

    let mut job_ids = Vec::with_capacity(jobs.len());
    let mut job_types_frequency = HashMap::new();

    let new_jobs: Vec<CreateJob> = jobs
        .into_iter()
        .map(|job| {
            job_ids.push(job.id);

            let mut run_at = Utc::now();

            match cron_parsed_to_time(&job.cron_expression, false) {
                Ok(time) => {
                    run_at = time;
                }
                Err(e) => {
                    error!(error = ?e);
                }
            }

            *job_types_frequency
                .entry(job.job_type.clone())
                .or_insert(0_u64) += 1;

            CreateJob {
                job_type: job.job_type,
                payload: job.payload,
                cron_expression: Some(job.cron_expression),
                status: JobStatus::Pending,
                priority: job.priority,
                max_retries: job.max_retries,
                created_at: Utc::now(),
                run_mode: RunMode::Recurring,
                run_at: run_at,
                parent_job_id: Some(job.parent_job_id.unwrap_or(job.id)),
            }
        })
        .collect();

    let inserted = bulk_insert_jobs(&mut *txn, new_jobs).await?;

    let updated = mark_recurring_jobs_as_rescheduled(&mut *txn, &job_ids).await?;
    info!(
        inserted = inserted,
        updated = updated,
        "Inserted recurring jobs and marked/updated the parent jobs as rescheduled to true"
    );

    txn.commit().await?;
    info!("job_types_frequency: {:?}", job_types_frequency);

    for (job_type, frequency) in job_types_frequency {
        state
            .metrics
            .cron_jobs_rescheduled
            .get_or_create(&JobType { job_type })
            .inc_by(frequency);
    }

    Ok(())
}
