use crate::common::errors::ProxyError;
use crate::modules::proxy::fetchable::Fetchable;
use crate::modules::proxy::sources::HttpFetcher;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AliasSource {
  aliases: HashMap<String, String>,
  http: Arc<HttpFetcher>,
  s3: Option<Arc<dyn Fetchable>>,
  local: Option<Arc<dyn Fetchable>>,
}

impl AliasSource {
  pub fn new(
    aliases: HashMap<String, String>,
    http: Arc<HttpFetcher>,
    s3: Option<Arc<dyn Fetchable>>,
    local: Option<Arc<dyn Fetchable>>,
  ) -> Self {
    Self {
      aliases,
      http,
      s3,
      local,
    }
  }

  fn resolve(&self, url: &str) -> Result<String, ProxyError> {
    let (scheme, raw_path) = url
      .split_once(':')
      .ok_or_else(|| ProxyError::InvalidParams("unrecognized URL format".to_string()))?;
    let base = self
      .aliases
      .get(scheme)
      .ok_or_else(|| ProxyError::InvalidParams(format!("unknown alias scheme: {scheme}")))?;
    let path = raw_path.trim_start_matches('/');
    Ok(format!("{}/{}", base.trim_end_matches('/'), path))
  }
}

#[async_trait::async_trait]
impl Fetchable for AliasSource {
  async fn fetch(&self, url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError> {
    let resolved = self.resolve(url)?;
    tracing::debug!(
      alias_url = url,
      resolved_url = resolved.as_str(),
      "alias resolved"
    );
    if resolved.starts_with("s3:/") {
      self
        .s3
        .as_ref()
        .ok_or_else(|| ProxyError::InvalidParams("S3 source not configured".to_string()))?
        .fetch(&resolved)
        .await
    } else if resolved.starts_with("local:/") {
      self
        .local
        .as_ref()
        .ok_or_else(|| ProxyError::InvalidParams("local source not configured".to_string()))?
        .fetch(&resolved)
        .await
    } else {
      self.http.fetch(&resolved).await
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::modules::security::allowlist::Allowlist;
  use wiremock::matchers::{method, path};
  use wiremock::{Mock, MockServer, ResponseTemplate};

  fn make_alias_source(aliases: Vec<(&str, &str)>) -> AliasSource {
    let http = Arc::new(
      HttpFetcher::new(10, 1_000_000, Arc::new(Allowlist::new(vec![])))
        .with_private_ip_check(false),
    );
    let map = aliases
      .into_iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect();
    AliasSource::new(map, http, None, None)
  }

  struct StubFetcher {
    response: Vec<u8>,
    content_type: Option<String>,
  }

  #[async_trait::async_trait]
  impl Fetchable for StubFetcher {
    async fn fetch(&self, _url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError> {
      Ok((self.response.clone(), self.content_type.clone()))
    }
  }

  fn make_alias_source_with_backends(
    aliases: Vec<(&str, &str)>,
    s3: Option<Arc<dyn Fetchable>>,
    local: Option<Arc<dyn Fetchable>>,
  ) -> AliasSource {
    let http = Arc::new(
      HttpFetcher::new(10, 1_000_000, Arc::new(Allowlist::new(vec![])))
        .with_private_ip_check(false),
    );
    let map = aliases
      .into_iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect();
    AliasSource::new(map, http, s3, local)
  }

  #[test]
  fn test_resolve_url_basic() {
    let source = make_alias_source(vec![("mycdn", "https://img.example.com")]);
    // Test URL rewriting logic directly
    let resolved = source.resolve("mycdn:/path/img.jpg").unwrap();
    assert_eq!(resolved, "https://img.example.com/path/img.jpg");
  }

  #[test]
  fn test_resolve_url_trailing_slash_on_base() {
    let source = make_alias_source(vec![("mycdn", "https://img.example.com/")]);
    let resolved = source.resolve("mycdn:/path/img.jpg").unwrap();
    assert_eq!(resolved, "https://img.example.com/path/img.jpg");
  }

  #[test]
  fn test_resolve_url_leading_slash_on_path() {
    let source = make_alias_source(vec![("mycdn", "https://img.example.com")]);
    let resolved = source.resolve("mycdn://path/img.jpg").unwrap();
    assert_eq!(resolved, "https://img.example.com/path/img.jpg");
  }

  #[test]
  fn test_resolve_url_no_leading_slash_on_path() {
    let source = make_alias_source(vec![("mycdn", "https://img.example.com")]);
    let resolved = source.resolve("mycdn:path/img.jpg").unwrap();
    assert_eq!(resolved, "https://img.example.com/path/img.jpg");
  }

  #[test]
  fn test_resolve_url_with_query_string() {
    let source = make_alias_source(vec![("mycdn", "https://img.example.com")]);
    let resolved = source.resolve("mycdn:/img.jpg?w=100&h=200").unwrap();
    assert_eq!(resolved, "https://img.example.com/img.jpg?w=100&h=200");
  }

  #[test]
  fn test_resolve_unknown_scheme_error() {
    let source = make_alias_source(vec![("mycdn", "https://img.example.com")]);
    let err = source.resolve("other:/img.jpg").unwrap_err();
    assert!(
      matches!(&err, ProxyError::InvalidParams(m) if m == "unknown alias scheme: other"),
      "unexpected: {:?}",
      err
    );
  }

  #[test]
  fn test_resolve_no_colon_slash_error() {
    let source = make_alias_source(vec![("mycdn", "https://img.example.com")]);
    let err = source.resolve("notaurl").unwrap_err();
    assert!(
      matches!(&err, ProxyError::InvalidParams(m) if m == "unrecognized URL format"),
      "unexpected: {:?}",
      err
    );
  }

  #[tokio::test]
  async fn test_fetch_dispatches_to_s3_when_resolved_url_starts_with_s3() {
    let s3_stub: Arc<dyn Fetchable> = Arc::new(StubFetcher {
      response: b"s3data".to_vec(),
      content_type: Some("image/png".to_string()),
    });
    let source =
      make_alias_source_with_backends(vec![("mybucket", "s3:/bucket-prefix")], Some(s3_stub), None);
    let (bytes, ct) = source.fetch("mybucket:/img.png").await.unwrap();
    assert_eq!(bytes, b"s3data");
    assert_eq!(ct, Some("image/png".to_string()));
  }

  #[tokio::test]
  async fn test_fetch_dispatches_to_local_when_resolved_url_starts_with_local() {
    let local_stub: Arc<dyn Fetchable> = Arc::new(StubFetcher {
      response: b"localdata".to_vec(),
      content_type: None,
    });
    let source =
      make_alias_source_with_backends(vec![("assets", "local:/uploads")], None, Some(local_stub));
    let (bytes, ct) = source.fetch("assets:/photo.jpg").await.unwrap();
    assert_eq!(bytes, b"localdata");
    assert!(ct.is_none());
  }

  #[tokio::test]
  async fn test_fetch_s3_alias_without_s3_configured_returns_error() {
    let source =
      make_alias_source_with_backends(vec![("mybucket", "s3:/bucket-prefix")], None, None);
    let err = source.fetch("mybucket:/img.png").await.unwrap_err();
    assert!(
      matches!(&err, ProxyError::InvalidParams(m) if m.contains("S3")),
      "unexpected: {:?}",
      err
    );
  }

  #[tokio::test]
  async fn test_fetch_local_alias_without_local_configured_returns_error() {
    let source = make_alias_source_with_backends(vec![("assets", "local:/uploads")], None, None);
    let err = source.fetch("assets:/photo.jpg").await.unwrap_err();
    assert!(
      matches!(&err, ProxyError::InvalidParams(m) if m.contains("local")),
      "unexpected: {:?}",
      err
    );
  }

  #[tokio::test]
  async fn test_fetch_resolves_and_fetches() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/path/img.jpg"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(b"imagedata".to_vec())
          .insert_header("content-type", "image/jpeg"),
      )
      .mount(&server)
      .await;

    let source = make_alias_source(vec![("mycdn", &server.uri())]);
    let (bytes, ct) = source.fetch("mycdn:/path/img.jpg").await.unwrap();
    assert_eq!(bytes, b"imagedata");
    assert_eq!(ct, Some("image/jpeg".to_string()));
  }
}
