use std::{
    process::{Child, Command},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use shared::config::load_supervisor_config;

fn main() {
    let config = load_supervisor_config("./config").expect("Config Error");

    let mut workers = Vec::with_capacity(config.workers as usize);
    for _ in 0..config.workers {
        workers.push(spawn_worker());
    }

    let process_terminated = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGTERM, process_terminated.clone())
        .expect("Cannot register a SIGTERM signal handler");
    signal_hook::flag::register(signal_hook::consts::SIGINT, process_terminated.clone())
        .expect("Cannot register a SIGINT signal handler");

    while !process_terminated.load(Ordering::Relaxed) {
        for worker in workers.iter_mut() {
            if let Ok(Some(status)) = worker.try_wait() {
                println!("Worker exited with status: {status}");

                *worker = spawn_worker();
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(config.poll_interval as u64));
    }

    for mut child in workers {
        match signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM) {
            Ok(_) => println!("Sent SIGTERM to Worker (PID: {:?})", child.id()),
            Err(e) => eprintln!("error: {:?}", e),
        }
        match child.wait() {
            Ok(_) => println!("Worker killed (PID: {:?})", child.id()),
            Err(e) => eprintln!("error: {:?}", e),
        }
    }
}

fn spawn_worker() -> Child {
    let child = Command::new("./target/release/worker")
        .spawn()
        .expect("Failed to spawn worker process");
    println!("Spawned Worker (PID: {:?})", child.id());
    child
}
