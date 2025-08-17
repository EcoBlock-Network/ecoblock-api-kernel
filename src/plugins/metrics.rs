use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder, IntCounterVec, Opts, Registry, HistogramVec, HistogramOpts};
use std::sync::Arc;

#[derive(Clone)]
pub struct MetricsPlugin {
    registry: Arc<Registry>,
    pub request_counter: Arc<IntCounterVec>,
    pub request_duration: Arc<HistogramVec>,
}

impl MetricsPlugin {
    pub fn new() -> Self {
        let registry = Registry::new();
        let ctr_opts = Opts::new("requests_total", "Total HTTP requests");
        let counter = IntCounterVec::new(ctr_opts, &["method", "path", "status"]).expect("counter");
        registry.register(Box::new(counter.clone())).ok();

        let hist_opts = HistogramOpts::new("request_duration_seconds", "HTTP request latencies in seconds");
        let histogram = HistogramVec::new(hist_opts, &["method", "path"]).expect("histogram");
        registry.register(Box::new(histogram.clone())).ok();

        // register process collector when available (platform/feature gated in prometheus crate)
        // register process collector only on Linux when the prometheus `process` feature is enabled
        #[cfg(target_os = "linux")]
        {
            let collector = prometheus::process_collector::ProcessCollector::for_self();
            registry.register(Box::new(collector)).ok();
        }

        MetricsPlugin {
            registry: Arc::new(registry),
            request_counter: Arc::new(counter),
            request_duration: Arc::new(histogram),
        }
    }

    pub fn router(&self) -> Router {
        let reg = self.registry.clone();
        Router::new().route("/", get(move || {
            let encoder = TextEncoder::new();
            let metric_families = reg.gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let body = String::from_utf8(buffer).unwrap();
            async move { (axum::http::StatusCode::OK, body) }
        }))
    }
}
