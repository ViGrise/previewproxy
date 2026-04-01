use crate::modules::health::dto::HealthResponse;
use tracing::info;

#[tracing::instrument]
pub async fn index(
  cache_memory_items: u64,
  cache_disk_bytes: u64,
  cache_disk_bytes_as_of: u64,
) -> HealthResponse {
  let response = HealthResponse {
    status: "ok".to_string(),
    cache_memory_items,
    cache_disk_bytes,
    cache_disk_bytes_as_of,
  };
  info!(status = %response.status, "health check result");
  response
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_health_index_returns_ok() {
    let result = index(0, 0, 0).await;
    assert_eq!(result.status, "ok");
  }

  #[tokio::test]
  async fn test_health_index_has_status_field() {
    let result = index(0, 0, 0).await;
    assert!(!result.status.is_empty());
  }
}
