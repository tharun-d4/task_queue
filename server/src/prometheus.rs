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

#[derive(Debug)]
pub struct Metrics {
    pub http_requests_total: Family<HttpLabel, Counter>,
    pub http_request_duration_seconds: Family<HttpLabel, Histogram>,
}

pub fn register_metrics() -> (Registry, Metrics) {
    let mut registry = Registry::default();

    let metrics = Metrics {
        http_requests_total: Family::default(),
        http_request_duration_seconds: Family::new_with_constructor(|| {
            Histogram::new([
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ])
        }),
    };

    registry.register(
        "http_requests_total",
        "Total HTTP requests",
        metrics.http_requests_total.clone(),
    );
    registry.register(
        "http_request_duration_seconds",
        "Request duration time in seconds",
        metrics.http_request_duration_seconds.clone(),
    );
    (registry, metrics)
}
