mod setup;

#[tokio::test]
async fn test_create_job() {
    let fixture = setup::TestFixture::new().await;

    let response = fixture
        .server
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
    response.assert_status_ok();
}
