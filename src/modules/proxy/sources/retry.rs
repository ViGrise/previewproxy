use crate::common::errors::ProxyError;
use crate::modules::metrics::Metrics;
use crate::modules::proxy::fetchable::Fetchable;
use std::sync::Arc;
use std::time::Duration;

/// Wraps any `Fetchable` and retries on transient connection errors.
///
/// Only retries when `ProxyError::is_connection_error()` returns true
/// (timeout, connection reset, broken pipe, etc.). All other errors
/// are returned immediately without retry.
pub struct RetryFetcher {
  inner: Arc<dyn Fetchable>,
  max_retries: u32,
  delay: Duration,
  metrics: Arc<Metrics>,
}

impl RetryFetcher {
  pub fn new(
    inner: Arc<dyn Fetchable>,
    max_retries: u32,
    delay_ms: u64,
    metrics: Arc<Metrics>,
  ) -> Self {
    Self {
      inner,
      max_retries,
      delay: Duration::from_millis(delay_ms),
      metrics,
    }
  }
}

#[async_trait::async_trait]
impl Fetchable for RetryFetcher {
  async fn fetch(&self, url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError> {
    let mut last_err = ProxyError::InternalError("no attempts made".to_string());
    for attempt in 0..=self.max_retries {
      match self.inner.fetch(url).await {
        Ok(result) => return Ok(result),
        Err(e) if e.is_connection_error() => {
          tracing::warn!(
            url = url,
            attempt = attempt,
            error = %e,
            "fetch failed with connection error, retrying"
          );
          self.metrics.fetch_retries_total.inc();
          last_err = e;
          if attempt < self.max_retries && !self.delay.is_zero() {
            tokio::time::sleep(self.delay).await;
          }
        }
        Err(e) => return Err(e),
      }
    }
    Err(last_err)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::errors::ProxyError;
  use crate::modules::metrics::Metrics;
  use std::sync::Arc;
  use std::sync::atomic::{AtomicU32, Ordering};

  fn test_metrics() -> Arc<Metrics> {
    Metrics::new("test_retry")
  }

  struct CountingFetcher {
    calls: Arc<AtomicU32>,
    errors: Vec<ProxyError>,
    success: Option<(Vec<u8>, Option<String>)>,
  }

  impl CountingFetcher {
    fn new(errors: Vec<ProxyError>, success: Option<(Vec<u8>, Option<String>)>) -> Arc<Self> {
      Arc::new(Self {
        calls: Arc::new(AtomicU32::new(0)),
        errors,
        success,
      })
    }
  }

  #[async_trait::async_trait]
  impl Fetchable for CountingFetcher {
    async fn fetch(&self, _url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError> {
      let i = self.calls.fetch_add(1, Ordering::SeqCst) as usize;
      if i < self.errors.len() {
        return Err(self.errors[i].clone());
      }
      match &self.success {
        Some(v) => Ok(v.clone()),
        None => Err(ProxyError::UpstreamNotFound),
      }
    }
  }

  #[tokio::test]
  async fn retries_on_timeout_then_succeeds() {
    let inner = CountingFetcher::new(
      vec![
        ProxyError::UpstreamConnectionError,
        ProxyError::UpstreamConnectionError,
      ],
      Some((b"ok".to_vec(), None)),
    );
    let calls = inner.calls.clone();
    let fetcher = RetryFetcher::new(inner, 3, 0, test_metrics());
    let result = fetcher.fetch("http://example.com").await;
    assert!(result.is_ok());
    assert_eq!(calls.load(Ordering::SeqCst), 3);
  }

  #[tokio::test]
  async fn does_not_retry_non_connection_errors() {
    let inner = CountingFetcher::new(
      vec![ProxyError::UpstreamNotFound],
      Some((b"ok".to_vec(), None)),
    );
    let calls = inner.calls.clone();
    let fetcher = RetryFetcher::new(inner, 3, 0, test_metrics());
    let result = fetcher.fetch("http://example.com").await;
    assert!(matches!(result, Err(ProxyError::UpstreamNotFound)));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn exhausts_retries_and_returns_last_error() {
    let inner = CountingFetcher::new(
      vec![
        ProxyError::UpstreamConnectionError,
        ProxyError::UpstreamConnectionError,
        ProxyError::UpstreamConnectionError,
        ProxyError::UpstreamConnectionError,
      ],
      None,
    );
    let calls = inner.calls.clone();
    let fetcher = RetryFetcher::new(inner, 3, 0, test_metrics());
    let result = fetcher.fetch("http://example.com").await;
    assert!(matches!(result, Err(ProxyError::UpstreamConnectionError)));
    assert_eq!(calls.load(Ordering::SeqCst), 4); // 1 initial + 3 retries
  }

  #[tokio::test]
  async fn zero_retries_makes_single_attempt() {
    let inner = CountingFetcher::new(vec![ProxyError::UpstreamConnectionError], None);
    let calls = inner.calls.clone();
    let fetcher = RetryFetcher::new(inner, 0, 0, test_metrics());
    let result = fetcher.fetch("http://example.com").await;
    assert!(matches!(result, Err(ProxyError::UpstreamConnectionError)));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }
}
