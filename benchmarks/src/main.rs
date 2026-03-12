use std::process::Command;

fn main() {
    println!("Begin benchmarking...");

    let build_server = Command::new("cargo")
        .args(["build", "--bin", "server"])
        .output()
        .expect("failed to compile server");

    println!("build_server: {:?}", build_server);

    let server_process = Command::new("./target/debug/server")
        .spawn()
        .expect("failed to spawn a server process");
    println!("server_process pid: {:?}", server_process.id());

    // kill the server process by sending SIGTERM
    Command::new("kill")
        .args(["-TERM", &server_process.id().to_string()])
        .output()
        .expect("Killing server process");
}
