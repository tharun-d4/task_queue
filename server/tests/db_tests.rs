use server::db::{
    models::{JobStats, JobStatsByJobType},
    queries,
};
use sqlx::PgPool;

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn job_stats_return_valid_stats(pool: PgPool) -> Result<(), sqlx::Error> {
    let stats = queries::get_job_stats(&pool).await?;

    assert_eq!(
        stats,
        JobStats {
            pending: 3,
            running: 1,
            completed: 0,
            failed: 0,
            cancelled: 0,
        }
    );
    Ok(())
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn job_stats_by_job_type_return_valid_stats(pool: PgPool) -> Result<(), sqlx::Error> {
    let stats = queries::get_job_stats_by_job_type(&pool).await?;

    assert_eq!(
        stats,
        vec![
            JobStatsByJobType {
                job_type: String::from("send_email"),
                pending: 2,
                running: 0,
                completed: 0,
                failed: 0,
                cancelled: 0,
            },
            JobStatsByJobType {
                job_type: String::from("send_webhook"),
                pending: 1,
                running: 1,
                completed: 0,
                failed: 0,
                cancelled: 0,
            }
        ]
    );
    Ok(())
}
