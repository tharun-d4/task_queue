use std::process::Command;

use tokio::time::Instant;

#[tokio::main]
async fn main() {
    println!("Begin benchmarking...");

    let build_server = Command::new("cargo")
        .args(["build", "--bin", "server"])
        .output()
        .expect("failed to compile server");

    println!("build_server: {:?}", build_server);

    let server_process = Command::new("./target/debug/server")
        .spawn()
        .expect("failed to spawn a server process");
    println!("Started Server (pid: {:?})", server_process.id());

    let build_worker = Command::new("cargo")
        .args(["build", "--bin", "worker"])
        .output()
        .expect("failed to compile worker");
    println!("build_worker: {:?}", build_worker);

    let supervisor_process = Command::new("cargo")
        .args(["run", "--bin", "worker_supervisor"])
        .spawn()
        .expect("failed to start worker supervisor");

    let req_client = reqwest::Client::new();

    const TOTAL_JOBS: u32 = 1000;
    let start = Instant::now();
    let mut success = 0;
    let mut error = 0;
    for _ in 0..TOTAL_JOBS {
        let response = req_client
            .post("http://127.0.0.1:8000/jobs")
            .json(&serde_json::json!({
                "job_type": "send_email",
                "payload": {
                    "to": "to_email@mail.com",
                    "from": "job_scheduler@mail.com",
                    "subject": "This is a sample load test / benchmark",
                    "body": "Yes this is just a sample load test / benchmark"
                },
                "priority": 10,
                "max_retries": 5,
            }))
            .send()
            .await;
        if let Ok(resp) = response
            && resp.status() == reqwest::StatusCode::CREATED
        {
            success += 1;
        } else {
            error += 1;
        }
    }
    println!("Submitted {} email jobs", TOTAL_JOBS);

    let end = start.elapsed();
    println!("Submission results: ");
    println!("Duration: {:.2}sec", end.as_secs_f64());
    println!("Successful: {success}");
    println!("Errors: {error}");
    println!(
        "Rate: {:.2} jobs/sec",
        TOTAL_JOBS as f64 / end.as_secs_f64()
    );

    // kill the server, worker supervisor processes by sending SIGTERM
    Command::new("kill")
        .args([
            "-TERM",
            &server_process.id().to_string(),
            &supervisor_process.id().to_string(),
        ])
        .output()
        .expect("Killing server process");
}
