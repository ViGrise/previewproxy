use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheEntry {
  pub bytes: Vec<u8>,
  pub content_type: String,
}

pub struct MemoryCache {
  pub inner: Cache<String, CacheEntry>,
}

impl MemoryCache {
  pub fn new(max_mb: u64, ttl: Duration) -> Self {
    let max_bytes = max_mb * 1024 * 1024;
    let cache = Cache::builder()
      .max_capacity(max_bytes)
      .time_to_live(ttl)
      .weigher(|k: &String, v: &CacheEntry| (k.len() + v.bytes.len()) as u32)
      .build();
    Self { inner: cache }
  }

  pub async fn get(&self, key: &str) -> Option<CacheEntry> {
    self.inner.get(key).await
  }

  pub async fn set(&self, key: String, entry: CacheEntry) {
    self.inner.insert(key, entry).await;
  }

  pub fn item_count(&self) -> u64 {
    self.inner.entry_count()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[tokio::test]
  async fn test_set_and_get() {
    let cache = MemoryCache::new(10, Duration::from_secs(60));
    let entry = CacheEntry {
      bytes: vec![1, 2, 3],
      content_type: "image/png".to_string(),
    };
    cache.set("key1".to_string(), entry.clone()).await;
    let result = cache.get("key1").await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().content_type, "image/png");
  }

  #[tokio::test]
  async fn test_miss() {
    let cache = MemoryCache::new(10, Duration::from_secs(60));
    assert!(cache.get("missing").await.is_none());
  }

  #[tokio::test]
  async fn test_item_count() {
    let cache = MemoryCache::new(10, Duration::from_secs(60));
    cache
      .set(
        "a".to_string(),
        CacheEntry {
          bytes: vec![0],
          content_type: "image/png".to_string(),
        },
      )
      .await;
    cache
      .set(
        "b".to_string(),
        CacheEntry {
          bytes: vec![0],
          content_type: "image/png".to_string(),
        },
      )
      .await;
    // Allow async eviction to process
    cache.inner.run_pending_tasks().await;
    assert_eq!(cache.item_count(), 2);
  }
}
