use axum::http::StatusCode;
use sqlx::PgPool;

use server::handlers::JobId;
use shared::db::models::Job;

mod test_server;

fn _sleep(secs: u64) {
    std::thread::sleep(std::time::Duration::from_secs(secs));
}

#[sqlx::test(migrations = "../migrations")]
async fn test_create_job(pool: PgPool) {
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
async fn test_get_job(pool: PgPool) {
    let server = test_server::build_test_server(pool);
    let create_job_response = server
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

    create_job_response.assert_status(StatusCode::CREATED);
    let job_id = create_job_response.json::<JobId>().job_id;

    let get_job_response = server.get(&format!("/jobs/{}", job_id)).await;
    get_job_response.assert_status_ok();

    let job = get_job_response.json::<Job>();
    assert_eq!(job_id, job.id);
}
