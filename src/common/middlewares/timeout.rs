use std::time::Duration;

use hyper::StatusCode;
use tower_http::timeout::TimeoutLayer;
use tracing::debug;

/// Layer that applies the Timeout middleware which apply a timeout to requests.
/// The default timeout value is set to 15 seconds.
/// When a request exceeds the timeout, tower-http returns 408 REQUEST_TIMEOUT automatically;
/// per-request timeout events should be observed via tracing spans on the handler side.
#[tracing::instrument]
pub fn timeout_layer() -> TimeoutLayer {
  let timeout_secs = 15u64;
  debug!(timeout_secs, "timeout middleware applied");
  TimeoutLayer::with_status_code(
    StatusCode::REQUEST_TIMEOUT,
    Duration::from_secs(timeout_secs),
  )
}
