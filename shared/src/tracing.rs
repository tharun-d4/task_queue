use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::fmt;

pub fn init_tracing(service_name: &str) -> non_blocking::WorkerGuard {
    let file_appender = rolling::daily("./logs", format!("{}.log", service_name));

    let (non_blocking_writer, guard) = non_blocking(file_appender);

    let format = fmt::format()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true);

    fmt()
        .event_format(format)
        .with_writer(non_blocking_writer)
        .init();

    guard
}
