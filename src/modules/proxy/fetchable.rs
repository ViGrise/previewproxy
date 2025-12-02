use crate::common::errors::ProxyError;

#[async_trait::async_trait]
pub trait Fetchable: Send + Sync {
  async fn fetch(&self, url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError>;
}
