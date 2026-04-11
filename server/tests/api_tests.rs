use axum::http::StatusCode;
use server::{
    db::models::{JobStats, JobStatsByJobType},
    handlers::JobStatsResponse,
};
use shared::db::models::Job;
use sqlx::PgPool;
use uuid::Uuid;

mod test_server;

#[sqlx::test(migrations = "../migrations")]
async fn create_job_with_valid_payload_returns_201(pool: PgPool) {
    let server = test_server::build_test_server(pool);

    let response = server
        .post("/jobs")
        .json(&serde_json::json!({
            "job_type": "send_email",
            "payload": {
                "to": "to_email@mail.com",
                "from": "job_scheduler@mail.com",
                "subject": "This is a sample test",
                "body": "Yes this is just a sample api test"
            },
            "priority": 10,
            "max_retries": 5,
        }))
        .await;
    response.assert_status(StatusCode::CREATED);
}

#[sqlx::test(migrations = "../migrations")]
async fn create_job_with_invalid_payload_returns_422(pool: PgPool) {
    let server = test_server::build_test_server(pool);

    // job_type and max_retries are missing in the request payload
    // the response status code should 422
    let response = server
        .post("/jobs")
        .json(&serde_json::json!({
            "payload": {
                "to": "to_email@mail.com",
                "from": "job_scheduler@mail.com",
                "subject": "This is a sample test",
                "body": "Yes this is just a sample api test"
            },
            "priority": 10,
        }))
        .await;
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn get_job_for_valid_job_id_returns_job(pool: PgPool) {
    let server = test_server::build_test_server(pool);

    let job_id = "019bfadc-28bb-781d-9d22-acf23fe50117"
        .parse::<Uuid>()
        .unwrap();

    let get_job_response = server.get(&format!("/jobs/{}", job_id)).await;
    get_job_response.assert_status_ok();

    let job = get_job_response.json::<Job>();
    assert_eq!(job_id, job.id);
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn get_job_for_invalid_job_id_returns_404(pool: PgPool) {
    let server = test_server::build_test_server(pool);

    // job_id is invalid as it is not present in DB
    // returns a 404 status code: Job not found error
    let job_id = "019bfadc-28bb-781d-9d22-acf23fe50116"
        .parse::<Uuid>()
        .unwrap();

    let get_job_response = server.get(&format!("/jobs/{}", job_id)).await;
    get_job_response.assert_status(StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn job_stats_returns_200(pool: PgPool) {
    let server = test_server::build_test_server(pool);

    let response = server.get("/jobs/stats").await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<JobStats>();
    assert_eq!(
        json,
        JobStats {
            pending: 3,
            running: 0,
            completed: 0,
            failed: 0,
        },
    )
}

#[sqlx::test(
    migrations = "../migrations",
    fixtures(path = "../../test_fixtures", scripts("jobs"))
)]
async fn job_stats_detailed_returns_200(pool: PgPool) {
    let server = test_server::build_test_server(pool);

    let response = server.get("/jobs/stats/detailed").await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<JobStatsResponse>();
    assert_eq!(
        json,
        JobStatsResponse {
            overall: JobStats {
                pending: 3,
                running: 0,
                completed: 0,
                failed: 0,
            },
            by_job_type: vec![
                JobStatsByJobType {
                    job_type: String::from("send_email"),
                    pending: 2,
                    running: 0,
                    completed: 0,
                    failed: 0,
                },
                JobStatsByJobType {
                    job_type: String::from("send_webhook"),
                    pending: 1,
                    running: 0,
                    completed: 0,
                    failed: 0,
                }
            ]
        }
    )
}
