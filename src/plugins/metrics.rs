use axum::{Router, routing::get};
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
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
        // use 'route' as a lower-cardinality label instead of raw path
        let counter =
            IntCounterVec::new(ctr_opts, &["method", "route", "status"]).expect("counter");
        registry.register(Box::new(counter.clone())).ok();

        let mut hist_opts = HistogramOpts::new(
            "request_duration_seconds",
            "HTTP request latencies in seconds",
        );
        // sensible buckets for HTTP latencies (seconds)
        hist_opts.buckets = vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];
        let histogram = HistogramVec::new(hist_opts, &["method", "route"]).expect("histogram");
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
        Router::new().route(
            "/",
            get(move || {
                let encoder = TextEncoder::new();
                let metric_families = reg.gather();
                let mut buffer = Vec::new();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                let body = String::from_utf8(buffer).unwrap();
                async move { (axum::http::StatusCode::OK, body) }
            }),
        )
    }
}
