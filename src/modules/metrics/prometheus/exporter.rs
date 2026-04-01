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

#[cfg(test)]
mod tests {
  use crate::modules::metrics::Metrics;
  use axum::http::StatusCode;
  use tower::ServiceExt;

  #[tokio::test]
  async fn test_metrics_endpoint_returns_200_with_text_plain() {
    let metrics = Metrics::new("");
    metrics.requests_total.inc();

    let app = crate::modules::metrics::prometheus::router(metrics);
    let req = axum::http::Request::builder()
      .uri("/metrics")
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("");
    assert!(
      ct.contains("text/plain"),
      "content-type should be text/plain, got: {ct}"
    );
  }

  #[tokio::test]
  async fn test_metrics_endpoint_contains_requests_total() {
    use http_body_util::BodyExt;

    let metrics = Metrics::new("");
    metrics.requests_total.inc();
    metrics.requests_total.inc();

    let app = crate::modules::metrics::prometheus::router(metrics);
    let req = axum::http::Request::builder()
      .uri("/metrics")
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let text = std::str::from_utf8(&body).unwrap();
    assert!(
      text.contains("requests_total 2"),
      "expected requests_total 2 in output:\n{text}"
    );
  }

  #[tokio::test]
  async fn test_metrics_namespace_prefix() {
    use http_body_util::BodyExt;

    let metrics = Metrics::new("myapp");
    metrics.requests_total.inc();

    let app = crate::modules::metrics::prometheus::router(metrics);
    let req = axum::http::Request::builder()
      .uri("/metrics")
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let text = std::str::from_utf8(&body).unwrap();
    assert!(
      text.contains("myapp_requests_total"),
      "expected myapp_requests_total in output:\n{text}"
    );
  }
}
