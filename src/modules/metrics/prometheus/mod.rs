pub mod exporter;

use crate::modules::metrics::Metrics;
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn router(metrics: Arc<Metrics>) -> Router {
  Router::new()
    .route("/metrics", get(exporter::handle_metrics))
    .with_state(metrics)
}
