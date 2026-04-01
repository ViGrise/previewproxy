use crate::modules::metrics::Metrics;
use axum::{
  extract::State,
  http::{HeaderValue, StatusCode, header},
  response::{IntoResponse, Response},
};
use prometheus::Encoder;
use std::sync::Arc;

pub async fn handle_metrics(State(metrics): State<Arc<Metrics>>) -> Response {
  let encoder = prometheus::TextEncoder::new();
  let metric_families = metrics.registry.gather();
  let mut buf = Vec::new();
  match encoder.encode(&metric_families, &mut buf) {
    Ok(()) => {
      let content_type = HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8");
      let mut headers = axum::http::HeaderMap::new();
      headers.insert(header::CONTENT_TYPE, content_type);
      (StatusCode::OK, headers, buf).into_response()
    }
    Err(e) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("failed to encode metrics: {e}"),
    )
      .into_response(),
  }
}
