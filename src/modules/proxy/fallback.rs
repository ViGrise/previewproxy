use base64::Engine;
use bytes::Bytes;
use std::sync::Arc;

pub struct FallbackImage {
  pub bytes: Bytes,
  pub content_type: String,
}

impl FallbackImage {
  pub async fn load(cfg: &crate::common::config::Configuration) -> Option<Arc<Self>> {
    let has_data = cfg.fallback_image_data.is_some();
    let has_path = cfg.fallback_image_path.is_some();
    let has_url = cfg.fallback_image_url.is_some();

    let count = [has_data, has_path, has_url].iter().filter(|&&v| v).count();
    if count == 0 {
      return None;
    }
    if count > 1 {
      tracing::warn!(
        "Multiple fallback image sources configured; using highest priority: data > path > url"
      );
    }

    let (bytes, content_type) = if let Some(data) = &cfg.fallback_image_data {
      let raw = base64::engine::general_purpose::STANDARD
        .decode(data)
        .unwrap_or_else(|e| panic!("PP_FALLBACK_IMAGE_DATA is not valid base64: {e}"));
      let ct = detect_content_type(&raw);
      (Bytes::from(raw), ct)
    } else if let Some(path) = &cfg.fallback_image_path {
      let raw = std::fs::read(path)
        .unwrap_or_else(|e| panic!("PP_FALLBACK_IMAGE_PATH '{path}' could not be read: {e}"));
      let ct = detect_content_type(&raw);
      (Bytes::from(raw), ct)
    } else {
      let url = cfg.fallback_image_url.as_deref().unwrap();
      let resp = reqwest::get(url)
        .await
        .unwrap_or_else(|e| panic!("PP_FALLBACK_IMAGE_URL '{url}' could not be fetched: {e}"));
      let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
      let raw = resp
        .bytes()
        .await
        .unwrap_or_else(|e| panic!("PP_FALLBACK_IMAGE_URL '{url}' body read failed: {e}"));
      (raw, ct)
    };

    Some(Arc::new(FallbackImage {
      bytes,
      content_type,
    }))
  }
}

fn detect_content_type(bytes: &[u8]) -> String {
  if bytes.starts_with(b"\x89PNG") {
    "image/png".to_string()
  } else if bytes.starts_with(b"\xff\xd8\xff") {
    "image/jpeg".to_string()
  } else if bytes.starts_with(b"GIF8") {
    "image/gif".to_string()
  } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
    "image/webp".to_string()
  } else if bytes.len() >= 12 && bytes.get(4..8) == Some(b"ftyp") {
    "image/avif".to_string()
  } else {
    "application/octet-stream".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::config::Configuration;
  use base64::Engine;

  fn base_cfg() -> Configuration {
    use std::collections::HashSet;
    use std::net::{Ipv4Addr, SocketAddr};
    Configuration {
      env: crate::common::config::Environment::Development,
      listen_address: SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080)),
      app_port: 8080,
      hmac_key: None,
      source_url_encryption_key: None,
      allowed_hosts: vec![],
      fetch_timeout_secs: 10,
      fetch_retry_count: 0,
      fetch_retry_delay_ms: 0,
      max_source_bytes: 1_000_000,
      cache_memory_max_mb: 16,
      cache_memory_ttl_secs: 60,
      cache_dir: "/tmp/test-fallback".to_string(),
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
      max_concurrent_requests: 256,
      input_disallow: HashSet::new(),
      output_disallow: HashSet::new(),
      transform_disallow: HashSet::new(),
      url_aliases: None,
      best_format: Default::default(),
      prometheus_bind: None,
      prometheus_namespace: String::new(),
      fallback_image_data: None,
      fallback_image_path: None,
      fallback_image_url: None,
      fallback_image_http_code: 200,
      fallback_image_ttl: None,
      ttl: 86400,
    }
  }

  // 1x1 red PNG in base64
  const PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwADhQGAWjR9awAAAABJRU5ErkJggg==";

  fn png_bytes() -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
      .decode(PNG_B64)
      .unwrap()
  }

  #[tokio::test]
  async fn test_load_none_when_no_source() {
    let cfg = base_cfg();
    let result = FallbackImage::load(&cfg).await;
    assert!(result.is_none());
  }

  #[tokio::test]
  async fn test_load_from_base64_data() {
    let mut cfg = base_cfg();
    cfg.fallback_image_data = Some(PNG_B64.to_string());
    let result = FallbackImage::load(&cfg).await.unwrap();
    assert_eq!(result.bytes.as_ref(), png_bytes().as_slice());
    assert_eq!(result.content_type, "image/png");
  }

  #[tokio::test]
  async fn test_load_from_path() {
    let path = "/tmp/previewproxy-test-fallback.png";
    std::fs::write(path, png_bytes()).unwrap();
    let mut cfg = base_cfg();
    cfg.fallback_image_path = Some(path.to_string());
    let result = FallbackImage::load(&cfg).await.unwrap();
    assert_eq!(result.bytes.as_ref(), png_bytes().as_slice());
    assert_eq!(result.content_type, "image/png");
  }

  #[tokio::test]
  async fn test_load_from_url() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_bytes(png_bytes())
          .insert_header("content-type", "image/png"),
      )
      .mount(&server)
      .await;
    let mut cfg = base_cfg();
    cfg.fallback_image_url = Some(server.uri());
    let result = FallbackImage::load(&cfg).await.unwrap();
    assert_eq!(result.bytes.as_ref(), png_bytes().as_slice());
    assert_eq!(result.content_type, "image/png");
  }

  #[tokio::test]
  async fn test_data_takes_priority_over_path_and_url() {
    let mut cfg = base_cfg();
    cfg.fallback_image_data = Some(PNG_B64.to_string());
    cfg.fallback_image_path = Some("/nonexistent/path.png".to_string());
    cfg.fallback_image_url = Some("https://example.com/fallback.png".to_string());
    // Should succeed using data without trying path or url
    let result = FallbackImage::load(&cfg).await.unwrap();
    assert!(!result.bytes.is_empty());
    assert_eq!(result.content_type, "image/png");
  }

  #[tokio::test]
  async fn test_path_takes_priority_over_url() {
    let path = "/tmp/previewproxy-test-fallback2.png";
    std::fs::write(path, png_bytes()).unwrap();
    let mut cfg = base_cfg();
    cfg.fallback_image_path = Some(path.to_string());
    cfg.fallback_image_url = Some("https://example.com/fallback.png".to_string());
    let result = FallbackImage::load(&cfg).await.unwrap();
    assert_eq!(result.bytes.as_ref(), png_bytes().as_slice());
  }
}
