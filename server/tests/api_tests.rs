use axum::http::StatusCode;
use sqlx::PgPool;

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
