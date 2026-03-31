use crate::common::config::Config;
use crate::common::errors::ProxyError;
use crate::modules::security::encryption;
use crate::modules::AppState;
use crate::modules::cache::manager::CacheHit;
use crate::modules::cache::memory::CacheEntry;
use crate::modules::proxy::{
  dto::{
    ProcessResult,
    params::{TransformParams, from_query},
  },
  service::ProxyService,
};
use axum::{
  Router,
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode, header},
  response::{IntoResponse, Response},
  routing::get,
};
use futures::StreamExt;
use std::collections::HashMap;
use tokio::sync::OwnedSemaphorePermit;

fn decrypt_url(key: Option<&Vec<u8>>, blob: &str) -> Result<String, ProxyError> {
  let key = key.ok_or_else(|| {
    ProxyError::InvalidParams("source URL encryption key not configured".to_string())
  })?;
  encryption::decrypt(key, blob).map_err(|e| ProxyError::InvalidParams(e.to_string()))
}

/// Registers the two proxy entry points:
/// - `GET /proxy?url=<image_url>&<params>` - query-string style
/// - `GET /<params>/<image_url>` - path style (params encoded in path prefix)
pub fn router() -> Router<AppState> {
  Router::new()
    .route("/proxy", get(handle_query))
    .route("/{*path}", get(handle_path))
}

/// Handles `GET /proxy?url=...` requests.
///
/// Acquires a concurrency permit before processing; returns 503 with
/// `Retry-After: 1` immediately if all permits are exhausted.
async fn handle_query(
  State(state): State<AppState>,
  Query(query): Query<HashMap<String, String>>,
) -> Response {
  let permit = match state.concurrency.clone().try_acquire_owned() {
    Ok(p) => p,
    Err(_) => {
      return (
        StatusCode::SERVICE_UNAVAILABLE,
        [(
          axum::http::header::HeaderName::from_static("retry-after"),
          "1",
        )],
        axum::body::Body::empty(),
      )
        .into_response();
    }
  };
  handle_query_inner(state, query, permit)
    .await
    .unwrap_or_else(|e| e.into_response())
}

/// Handles `GET /<params>/<image_url>` requests.
///
/// Path params are parsed from the URL prefix; any additional query-string
/// params are merged in (query-string wins on conflicts). Acquires a
/// concurrency permit with the same 503 behaviour as `handle_query`.
async fn handle_path(
  State(state): State<AppState>,
  Path(path): Path<String>,
  Query(query): Query<HashMap<String, String>>,
) -> Response {
  let permit = match state.concurrency.clone().try_acquire_owned() {
    Ok(p) => p,
    Err(_) => {
      return (
        StatusCode::SERVICE_UNAVAILABLE,
        [(
          axum::http::header::HeaderName::from_static("retry-after"),
          "1",
        )],
        axum::body::Body::empty(),
      )
        .into_response();
    }
  };
  handle_path_inner(state, path, query, permit)
    .await
    .unwrap_or_else(|e| e.into_response())
}

async fn handle_query_inner(
  state: AppState,
  query: HashMap<String, String>,
  permit: OwnedSemaphorePermit,
) -> Result<Response, ProxyError> {
  let raw_url = query
    .get("url")
    .cloned()
    .ok_or_else(|| ProxyError::InvalidParams("missing `url` query param".to_string()))?;
  // Presence of `enc` key (any value) signals the URL is encrypted.
  let url = if query.contains_key("enc") {
    decrypt_url(state.cfg.source_url_encryption_key.as_ref(), &raw_url)?
  } else {
    raw_url
  };
  let params = from_query(&query)?;
  let service = ProxyService::new(&state);
  let result = service.process(params, url, permit).await?;
  Ok(build_response(result, &state.cfg))
}

async fn handle_path_inner(
  state: AppState,
  path: String,
  query: HashMap<String, String>,
  permit: OwnedSemaphorePermit,
) -> Result<Response, ProxyError> {
  let (mut params, raw_url) = TransformParams::from_path(&path)?;
  let url = if raw_url.starts_with("enc/") {
    let blob = &raw_url["enc/".len()..];
    decrypt_url(state.cfg.source_url_encryption_key.as_ref(), blob)?
  } else {
    raw_url
  };
  if !query.is_empty() {
    let query_params = from_query(&query)?;
    params.merge_from(query_params);
  }
  let svc = ProxyService::new(&state);
  let result = svc.process(params, url, permit).await?;
  Ok(build_response(result, &state.cfg))
}

/// Converts a `ProcessResult` into an HTTP response.
/// Cached results get `Cache-Control` and `X-Cache: HIT-L1/HIT-L2` headers.
/// Streamed results get `X-Cache: MISS` and body is forwarded as a chunked stream.
fn build_response(result: ProcessResult, cfg: &Config) -> Response {
  match result {
    ProcessResult::Cached(entry, hit) => build_cached_response(entry, hit, cfg),
    ProcessResult::Stream { body, content_type } => {
      let ct: axum::http::HeaderValue = content_type
        .parse()
        .unwrap_or_else(|_| "application/octet-stream".parse().unwrap());
      let mut headers = axum::http::HeaderMap::new();
      headers.insert(axum::http::header::CONTENT_TYPE, ct);
      headers.insert("x-cache", "MISS".parse().unwrap());
      let mapped =
        body.map(|r| r.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>));
      (headers, axum::body::Body::from_stream(mapped)).into_response()
    }
  }
}

fn build_cached_response(entry: CacheEntry, hit: CacheHit, cfg: &Config) -> Response {
  let x_cache = match hit {
    CacheHit::L1 => "HIT-L1",
    CacheHit::L2 => "HIT-L2",
    CacheHit::Miss => "MISS",
  };
  let content_length = entry.bytes.len();
  let cache_control = format!("public, max-age={}", cfg.cache_disk_ttl_secs);

  let mut headers = HeaderMap::new();
  let ct_value = entry
    .content_type
    .parse()
    .unwrap_or_else(|_| "application/octet-stream".parse().unwrap());
  headers.insert(header::CONTENT_TYPE, ct_value);
  headers.insert(header::CONTENT_LENGTH, content_length.into());
  headers.insert(header::CACHE_CONTROL, cache_control.parse().unwrap());
  headers.insert("x-cache", x_cache.parse().unwrap());

  (headers, entry.bytes).into_response()
}

#[cfg(test)]
mod concurrency_tests {
  use crate::common::config::Configuration;
  use crate::modules::AppState;
  use crate::modules::cache::manager::CacheManager;
  use crate::modules::proxy::sources::http::HttpFetcher;
  use crate::modules::security::allowlist::Allowlist;
  use axum::http::StatusCode;
  use std::net::{Ipv4Addr, SocketAddr};
  use std::sync::Arc;
  use tokio::sync::Semaphore;
  use tower::ServiceExt;

  fn make_state(permits: usize) -> AppState {
    let cfg = Arc::new(Configuration {
      env: crate::common::config::Environment::Development,
      listen_address: SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080)),
      app_port: 8080,
      hmac_key: None,
      source_url_encryption_key: None,
      allowed_hosts: vec![],
      fetch_timeout_secs: 10,
      max_source_bytes: 1_000_000,
      cache_memory_max_mb: 16,
      cache_memory_ttl_secs: 60,
      cache_dir: "/tmp/previewproxy-ctrl-test".to_string(),
      cache_disk_ttl_secs: 60,
      cache_disk_max_mb: None,
      cache_cleanup_interval_secs: 600,
      s3_enabled: false,
      s3_bucket: None,
      s3_region: "us-east-1".to_string(),
      s3_access_key_id: None,
      s3_secret_access_key: None,
      s3_endpoint: None,
      local_enabled: false,
      local_base_dir: None,
      ffmpeg_path: "ffmpeg".to_string(),
      ffprobe_path: "ffprobe".to_string(),
      cors_allow_origin: vec!["*".to_string()],
      cors_max_age_secs: 600,
      max_concurrent_requests: permits,
      input_disallow: std::collections::HashSet::new(),
      output_disallow: std::collections::HashSet::new(),
      transform_disallow: std::collections::HashSet::new(),
      url_aliases: None,
      best_format: Default::default(),
    });
    let http = Arc::new(
      HttpFetcher::new(10, 1_000_000, Arc::new(Allowlist::new(vec![])))
        .with_private_ip_check(false),
    );
    AppState {
      cache: CacheManager::new(&cfg),
      fetcher: http.clone(),
      http_fetcher: http,
      concurrency: Arc::new(Semaphore::new(permits)),
      cfg,
    }
  }

  fn make_state_with_enc_key(permits: usize, enc_key: Option<Vec<u8>>) -> AppState {
    let base = make_state(permits);
    let mut cfg = (*base.cfg).clone();
    cfg.source_url_encryption_key = enc_key;
    AppState {
      cfg: std::sync::Arc::new(cfg),
      ..base
    }
  }

  #[tokio::test]
  async fn test_path_encrypted_url_decrypts_and_proxies() {
    use http_body_util::BodyExt;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(vec![1u8; 10])
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;

    let key = b"01234567890123456789012345678901".to_vec(); // 32 bytes
    let blob = crate::modules::security::encryption::encrypt(&key, &server.uri()).unwrap();
    let state = make_state_with_enc_key(256, Some(key));
    let app = crate::modules::router(state);

    let req = axum::http::Request::builder()
      .uri(format!("/enc/{blob}"))
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let _ = resp.into_body().collect().await.unwrap();
  }

  #[tokio::test]
  async fn test_query_encrypted_url_decrypts_and_proxies() {
    use http_body_util::BodyExt;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(vec![1u8; 10])
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;

    let key = b"01234567890123456789012345678901".to_vec();
    let blob = crate::modules::security::encryption::encrypt(&key, &server.uri()).unwrap();
    let state = make_state_with_enc_key(256, Some(key));
    let app = crate::modules::router(state);

    let encoded_blob = urlencoding::encode(&blob).to_string();
    let req = axum::http::Request::builder()
      .uri(format!("/proxy?url={encoded_blob}&enc=1"))
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let _ = resp.into_body().collect().await.unwrap();
  }

  #[tokio::test]
  async fn test_encrypted_url_no_key_returns_400() {
    let state = make_state_with_enc_key(256, None);
    let app = crate::modules::router(state);
    let req = axum::http::Request::builder()
      .uri("/enc/someblob")
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn test_encrypted_url_bad_blob_returns_400() {
    let key = b"01234567890123456789012345678901".to_vec();
    let state = make_state_with_enc_key(256, Some(key));
    let app = crate::modules::router(state);
    let req = axum::http::Request::builder()
      .uri("/enc/!!!notvalidbase64!!!")
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn test_query_enc_flag_no_key_returns_400() {
    let state = make_state_with_enc_key(256, None);
    let app = crate::modules::router(state);
    let req = axum::http::Request::builder()
      .uri("/proxy?url=someblob&enc=1")
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn test_503_when_semaphore_exhausted() {
    let state = AppState {
      concurrency: Arc::new(Semaphore::new(0)), // 0 permits
      ..make_state(1)
    };
    let app = crate::modules::router(state);
    let req = axum::http::Request::builder()
      .uri("/proxy?url=https://example.com/img.jpg")
      .body(axum::body::Body::empty())
      .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
      response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok()),
      Some("1")
    );
  }

  #[tokio::test]
  async fn test_permit_restored_after_buffered_request() {
    let sem = Arc::new(Semaphore::new(1));
    let state = AppState {
      concurrency: sem.clone(),
      ..make_state(1)
    };
    assert_eq!(sem.available_permits(), 1);
    let app = crate::modules::router(state);
    let req = axum::http::Request::builder()
      .uri("/proxy?url=https://0.0.0.0/img.jpg") // will fail fast (HostNotAllowed or connect error)
      .body(axum::body::Body::empty())
      .unwrap();
    let _ = app.oneshot(req).await.unwrap();
    assert_eq!(sem.available_permits(), 1);
  }

  #[tokio::test]
  async fn test_permit_held_during_stream_released_after_exhaustion() {
    use http_body_util::BodyExt;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(vec![1u8; 20])
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;

    let sem = Arc::new(Semaphore::new(1));
    let state = AppState {
      concurrency: sem.clone(),
      ..make_state(1)
    };
    assert_eq!(sem.available_permits(), 1);

    let url = format!("/proxy?url={}", urlencoding::encode(&server.uri()));
    let app = crate::modules::router(state);
    let req = axum::http::Request::builder()
      .uri(&url)
      .body(axum::body::Body::empty())
      .unwrap();

    let resp = app.oneshot(req).await.unwrap();

    let _ = resp.into_body().collect().await.unwrap();

    assert_eq!(
      sem.available_permits(),
      1,
      "permit must be released after stream body is consumed"
    );
  }

  #[tokio::test]
  async fn test_streaming_x_cache_miss_header() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(vec![1u8; 10])
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;
    let state = make_state(256);
    let url = format!("/proxy?url={}", urlencoding::encode(&server.uri()));
    let app = crate::modules::router(state);
    let req = axum::http::Request::builder()
      .uri(&url)
      .body(axum::body::Body::empty())
      .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
      resp.headers().get("x-cache").and_then(|v| v.to_str().ok()),
      Some("MISS")
    );
  }
}
