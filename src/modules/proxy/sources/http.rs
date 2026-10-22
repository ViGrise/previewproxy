use crate::common::errors::ProxyError;
use crate::modules::proxy::fetchable::Fetchable;
use crate::modules::security::allowlist::{Allowlist, is_private_ip};
use futures::StreamExt;
use reqwest::{
  Client,
  dns::{Addrs, Name, Resolve, Resolving},
};
use std::{error::Error, sync::Arc, time::Duration};

#[derive(Debug, thiserror::Error)]
enum FetchPolicyError {
  #[error("host not allowed")]
  HostNotAllowed,
  #[error("too many redirects")]
  TooManyRedirects,
}

struct PublicResolver;

impl Resolve for PublicResolver {
  fn resolve(&self, name: Name) -> Resolving {
    Box::pin(async move {
      let addresses: Vec<_> = tokio::net::lookup_host((name.as_str(), 0)).await?.collect();
      if addresses.is_empty() || addresses.iter().any(|addr| is_private_ip(addr.ip())) {
        return Err(Box::new(FetchPolicyError::HostNotAllowed) as Box<dyn Error + Send + Sync>);
      }
      Ok(Box::new(addresses.into_iter()) as Addrs)
    })
  }
}

fn private_literal(url: &url::Url) -> bool {
  match url.host() {
    Some(url::Host::Ipv4(ip)) => is_private_ip(ip.into()),
    Some(url::Host::Ipv6(ip)) => is_private_ip(ip.into()),
    _ => false,
  }
}

fn map_send_error(error: reqwest::Error) -> ProxyError {
  let mut source: Option<&(dyn Error + 'static)> = Some(&error);
  while let Some(current) = source {
    if let Some(policy) = current.downcast_ref::<FetchPolicyError>() {
      return match policy {
        FetchPolicyError::HostNotAllowed => ProxyError::HostNotAllowed,
        FetchPolicyError::TooManyRedirects => ProxyError::TooManyRedirects,
      };
    }
    source = current.source();
  }
  if error.is_timeout() {
    ProxyError::UpstreamTimeout
  } else {
    ProxyError::UpstreamConnectionError
  }
}

/// HTTP/HTTPS source fetcher with SSRF protection, allowlist enforcement,
/// redirect validation, and body size limiting.
///
/// When `check_private_ips` is set, the DNS resolver rejects private and
/// non-routable addresses at connection time, which covers every redirect hop,
/// and private IP literals are rejected before each request. Development mode
/// turns the check off so loopback test servers stay reachable.
pub struct HttpFetcher {
  client: Client,
  trusted_client: Option<Client>,
  max_bytes: u64,
  timeout_secs: u64,
  allowlist: Arc<Allowlist>,
  check_private_ips: bool,
}

#[async_trait::async_trait]
impl Fetchable for HttpFetcher {
  async fn fetch(&self, url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError> {
    self.fetch_url(url).await
  }
}

impl HttpFetcher {
  pub fn with_private_ip_check(mut self, check: bool) -> Self {
    self.check_private_ips = check;
    self.client = Self::build_client(self.timeout_secs, self.allowlist.clone(), check);
    self
  }

  pub fn with_trusted_first_hop(mut self) -> Self {
    self.trusted_client = Some(
      Client::builder()
        .timeout(Duration::from_secs(self.timeout_secs))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("ViGrise-PreviewProxy/1.0")
        .build()
        .expect("valid HTTP client"),
    );
    self
  }

  pub fn new(
    timeout_secs: u64,
    max_bytes: u64,
    allowlist: Arc<Allowlist>,
    check_private_ips: bool,
  ) -> Self {
    Self {
      client: Self::build_client(timeout_secs, allowlist.clone(), check_private_ips),
      trusted_client: None,
      max_bytes,
      timeout_secs,
      allowlist,
      check_private_ips,
    }
  }

  fn build_client(timeout_secs: u64, allowlist: Arc<Allowlist>, check: bool) -> Client {
    let policy = reqwest::redirect::Policy::custom(move |attempt| {
      if attempt.previous().len() >= 5 {
        return attempt.error(FetchPolicyError::TooManyRedirects);
      }
      let url = attempt.url();
      if !allowlist.is_allowed(url.host_str().unwrap_or("")) || (check && private_literal(url)) {
        return attempt.error(FetchPolicyError::HostNotAllowed);
      }
      attempt.follow()
    });
    let mut builder = Client::builder()
      .timeout(Duration::from_secs(timeout_secs))
      .redirect(policy)
      .user_agent("ViGrise-PreviewProxy/1.0");
    if check {
      // Direct connections keep address validation on the connection's DNS path.
      builder = builder.no_proxy().dns_resolver(Arc::new(PublicResolver));
    }
    builder.build().expect("valid HTTP client")
  }

  async fn send_checked(&self, url: &str) -> Result<reqwest::Response, ProxyError> {
    let parsed =
      url::Url::parse(url).map_err(|_| ProxyError::InvalidParams("invalid URL".into()))?;
    let resp = if let Some(client) = &self.trusted_client {
      let first = client.get(parsed).send().await.map_err(map_send_error)?;
      if first.status().is_redirection() {
        if let Some(location) = first.headers().get(reqwest::header::LOCATION) {
          let next = first
            .url()
            .join(
              location
                .to_str()
                .map_err(|_| ProxyError::UpstreamConnectionError)?,
            )
            .map_err(|_| ProxyError::UpstreamConnectionError)?;
          if self.check_private_ips && private_literal(&next) {
            return Err(ProxyError::HostNotAllowed);
          }
          if !self.allowlist.is_allowed(next.host_str().unwrap_or("")) {
            return Err(ProxyError::HostNotAllowed);
          }
          self.client.get(next).send().await.map_err(map_send_error)?
        } else {
          first
        }
      } else {
        first
      }
    } else {
      if !self.allowlist.is_allowed(parsed.host_str().unwrap_or(""))
        || (self.check_private_ips && private_literal(&parsed))
      {
        return Err(ProxyError::HostNotAllowed);
      }
      self
        .client
        .get(parsed)
        .send()
        .await
        .map_err(map_send_error)?
    };
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
      return Err(ProxyError::UpstreamNotFound);
    }
    if !resp.status().is_success() {
      return Err(ProxyError::UpstreamBadStatus);
    }
    if resp
      .content_length()
      .is_some_and(|len| len > self.max_bytes)
    {
      return Err(ProxyError::SourceTooLarge);
    }
    Ok(resp)
  }

  async fn fetch_url(&self, url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError> {
    let resp = self.send_checked(url).await?;
    let content_type = resp
      .headers()
      .get(reqwest::header::CONTENT_TYPE)
      .and_then(|v| v.to_str().ok())
      .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());
    let mut stream = resp.bytes_stream();
    let mut buf = Vec::new();
    while let Some(chunk) = stream.next().await {
      let chunk = chunk.map_err(map_send_error)?;
      if (buf.len() as u64).saturating_add(chunk.len() as u64) > self.max_bytes {
        return Err(ProxyError::SourceTooLarge);
      }
      buf.extend_from_slice(&chunk);
    }
    Ok((buf, content_type))
  }

  /// Returns the reqwest::Response after header checks, without reading the body.
  /// Caller is responsible for reading or streaming the body.
  pub async fn fetch_streaming(&self, url: &str) -> Result<reqwest::Response, ProxyError> {
    self.send_checked(url).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::modules::security::allowlist::Allowlist;
  use std::sync::Arc;
  use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

  #[tokio::test]
  async fn security_private_literals_and_dns() {
    let fetcher = HttpFetcher::new(2, 1000, Arc::new(Allowlist::new(vec![])), true);
    for url in [
      "http://127.0.0.1",
      "http://[::1]",
      "http://[::ffff:127.0.0.1]",
      "http://localhost",
    ] {
      assert!(
        matches!(fetcher.fetch(url).await, Err(ProxyError::HostNotAllowed)),
        "{url}"
      );
    }
    assert!(fetcher.fetch("http://invalid.invalid").await.is_err());
  }

  #[tokio::test]
  async fn security_redirect_private_and_upstream_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(ResponseTemplate::new(302).insert_header("location", "http://[::1]/secret"))
      .mount(&server)
      .await;
    let fetcher =
      HttpFetcher::new(2, 1000, Arc::new(Allowlist::new(vec![])), true).with_trusted_first_hop();
    assert!(matches!(
      fetcher.fetch(&server.uri()).await,
      Err(ProxyError::HostNotAllowed)
    ));
    server.reset().await;
    Mock::given(method("GET"))
      .respond_with(ResponseTemplate::new(503))
      .mount(&server)
      .await;
    use axum::response::IntoResponse;
    let error = open_fetcher().fetch(&server.uri()).await.unwrap_err();
    assert_eq!(
      error.into_response().status(),
      axum::http::StatusCode::BAD_GATEWAY
    );
  }

  #[tokio::test]
  async fn security_redirect_policy_errors() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(ResponseTemplate::new(302).insert_header("location", server.uri().as_str()))
      .mount(&server)
      .await;
    assert!(matches!(
      open_fetcher().fetch(&server.uri()).await,
      Err(ProxyError::TooManyRedirects)
    ));
    server.reset().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(302).insert_header("location", "http://denied.example/image.png"),
      )
      .mount(&server)
      .await;
    let fetcher = HttpFetcher::new(
      2,
      1000,
      Arc::new(Allowlist::new(vec!["127.0.0.1".into()])),
      true,
    )
    .with_private_ip_check(false);
    assert!(matches!(
      fetcher.fetch(&server.uri()).await,
      Err(ProxyError::HostNotAllowed)
    ));
  }

  fn open_fetcher() -> HttpFetcher {
    HttpFetcher::new(10, 1_000_000, Arc::new(Allowlist::new(vec![])), true)
      .with_private_ip_check(false)
  }

  #[tokio::test]
  async fn test_fetch_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(vec![0u8; 10])
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;
    let fetcher = open_fetcher();
    let (bytes, ct) = fetcher.fetch(&server.uri()).await.unwrap();
    assert_eq!(bytes.len(), 10);
    assert_eq!(ct, Some("image/png".to_string()));
  }

  #[tokio::test]
  async fn test_fetch_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(ResponseTemplate::new(404))
      .mount(&server)
      .await;
    let result = open_fetcher().fetch(&server.uri()).await;
    assert!(matches!(result, Err(ProxyError::UpstreamNotFound)));
  }

  #[tokio::test]
  async fn test_fetch_too_large() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(vec![0u8; 200])
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;
    let fetcher =
      HttpFetcher::new(10, 50, Arc::new(Allowlist::new(vec![])), true).with_private_ip_check(false);
    let result = fetcher.fetch(&server.uri()).await;
    assert!(matches!(result, Err(ProxyError::SourceTooLarge)));
  }

  #[tokio::test]
  async fn test_no_content_type_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 5]))
      .mount(&server)
      .await;
    let (bytes, ct) = open_fetcher().fetch(&server.uri()).await.unwrap();
    assert_eq!(bytes.len(), 5);
    assert!(ct.is_none());
  }

  #[tokio::test]
  async fn test_fetch_streaming_returns_200_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(vec![0u8; 20])
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;
    let fetcher = open_fetcher();
    let resp = fetcher.fetch_streaming(&server.uri()).await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let ct = resp
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("");
    assert!(ct.contains("image/png"));
  }

  #[tokio::test]
  async fn test_fetch_streaming_404_is_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(ResponseTemplate::new(404))
      .mount(&server)
      .await;
    let result = open_fetcher().fetch_streaming(&server.uri()).await;
    assert!(matches!(result, Err(ProxyError::UpstreamNotFound)));
  }
}
