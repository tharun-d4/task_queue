use prometheus_client::{
    encoding::{EncodeLabelSet, EncodeLabelValue},
    metrics::{counter::Counter, family::Family, histogram::Histogram},
    registry::Registry,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HttpLabel {
    pub method: HttpMethod,
    pub path: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct JobType {
    pub job_type: String,
}

#[derive(Debug)]
pub struct Metrics {
    pub http_requests: Family<HttpLabel, Counter>,
    pub http_request_duration_seconds: Family<HttpLabel, Histogram>,

    pub jobs_submitted: Family<JobType, Counter>,
    pub lease_recovered_jobs: Family<JobType, Counter>,
    pub cron_jobs_rescheduled: Family<JobType, Counter>,
}

pub fn register_metrics() -> (Registry, Metrics) {
    let mut registry = Registry::default();

    let metrics = Metrics {
        http_requests: Family::default(),
        http_request_duration_seconds: Family::new_with_constructor(|| {
            Histogram::new([
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ])
        }),
        jobs_submitted: Family::default(),
        lease_recovered_jobs: Family::default(),
        cron_jobs_rescheduled: Family::default(),
    };

    registry.register(
        "http_requests",
        "Total HTTP requests",
        metrics.http_requests.clone(),
    );
    registry.register(
        "http_request_duration_seconds",
        "Request duration time in seconds",
        metrics.http_request_duration_seconds.clone(),
    );
    registry.register(
        "jobs_submitted",
        "Total Jobs Submitted",
        metrics.jobs_submitted.clone(),
    );
    registry.register(
        "lease_recovered_jobs",
        "Total Jobs whose lease is recovered",
        metrics.lease_recovered_jobs.clone(),
    );
    registry.register(
        "cron_jobs_rescheduled",
        "Total Cron Jobs Rescheduled",
        metrics.cron_jobs_rescheduled.clone(),
    );
    (registry, metrics)
}
