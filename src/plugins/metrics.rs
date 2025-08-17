use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder, IntCounterVec, Opts, Registry};
use std::sync::Arc;

#[derive(Clone)]
pub struct MetricsPlugin {
    registry: Arc<Registry>,
    pub request_counter: Arc<IntCounterVec>,
}

impl MetricsPlugin {
    pub fn new() -> Self {
        let registry = Registry::new();
        let opts = Opts::new("requests_total", "Total HTTP requests");
        let counter = IntCounterVec::new(opts, &["method", "path", "status"]).expect("counter");
        registry.register(Box::new(counter.clone())).ok();
        MetricsPlugin { registry: Arc::new(registry), request_counter: Arc::new(counter) }
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
