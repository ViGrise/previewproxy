use crate::modules::cli::dto::Commands;
use clap::Parser;

/// ViGrise PreviewProxy - on-the-fly image proxy and transformer
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Path to a .env file to load (overrides default .env lookup)
  #[arg(long, value_name = "PATH", global = true)]
  pub env_file: Option<String>,

  /// Server port [env: PP_PORT]
  #[arg(short, long, env = "PP_PORT", default_value = "8080")]
  pub port: u16,

  /// Environment: development or production [env: PP_APP_ENV]
  #[arg(short = 'E', long, env = "PP_APP_ENV", default_value = "development")]
  pub env: String,

  /// General response TTL in seconds [env: PP_TTL]
  #[arg(long, env = "PP_TTL", default_value_t = 86400u64)]
  pub ttl: u64,

  /// HMAC signing key (leave empty to disable) [env: PP_HMAC_KEY]
  #[arg(short = 'k', long, env = "PP_HMAC_KEY")]
  pub hmac_key: Option<String>,

  /// Comma-separated allowed upstream hosts (empty = allow all) [env: PP_ALLOWED_HOSTS]
  #[arg(short = 'a', long, env = "PP_ALLOWED_HOSTS", default_value = "")]
  pub allowed_hosts: String,

  /// Upstream fetch timeout in seconds [env: PP_FETCH_TIMEOUT_SECS]
  #[arg(short = 't', long, env = "PP_FETCH_TIMEOUT_SECS", default_value = "10")]
  pub fetch_timeout_secs: u64,

  /// Maximum source image size in bytes [env: PP_MAX_SOURCE_BYTES]
  #[arg(
    short = 's',
    long,
    env = "PP_MAX_SOURCE_BYTES",
    default_value = "20971520"
  )]
  pub max_source_bytes: u64,

  /// L1 in-memory cache size in MB [env: PP_CACHE_MEMORY_MAX_MB]
  #[arg(long, env = "PP_CACHE_MEMORY_MAX_MB", default_value = "256")]
  pub cache_memory_max_mb: u64,

  /// L1 in-memory cache TTL in seconds [env: PP_CACHE_MEMORY_TTL_SECS]
  #[arg(long, env = "PP_CACHE_MEMORY_TTL_SECS", default_value = "3600")]
  pub cache_memory_ttl_secs: u64,

  /// L2 disk cache directory [env: PP_CACHE_DIR]
  #[arg(
    short = 'D',
    long,
    env = "PP_CACHE_DIR",
    default_value = "/tmp/previewproxy"
  )]
  pub cache_dir: String,

  /// L2 disk cache TTL in seconds [env: PP_CACHE_DISK_TTL_SECS]
  #[arg(long, env = "PP_CACHE_DISK_TTL_SECS", default_value = "86400")]
  pub cache_disk_ttl_secs: u64,

  /// L2 disk cache max size in MB (empty = unlimited) [env: PP_CACHE_DISK_MAX_MB]
  #[arg(long, env = "PP_CACHE_DISK_MAX_MB", default_value = "")]
  pub cache_disk_max_mb: String,

  /// Cache cleanup interval in seconds [env: PP_CACHE_CLEANUP_INTERVAL_SECS]
  #[arg(long, env = "PP_CACHE_CLEANUP_INTERVAL_SECS", default_value = "600")]
  pub cache_cleanup_interval_secs: u64,

  /// Path to the ffmpeg binary [env: PP_FFMPEG_PATH]
  #[arg(long, env = "PP_FFMPEG_PATH", default_value = "ffmpeg")]
  pub ffmpeg_path: String,

  /// Path to the ffprobe binary (defaults to ffprobe in same dir as ffmpeg) [env: PP_FFPROBE_PATH]
  #[arg(long, env = "PP_FFPROBE_PATH", default_value = "")]
  pub ffprobe_path: String,

  /// Comma-separated allowed CORS origins; * = allow all [env: PP_CORS_ALLOW_ORIGIN]
  #[arg(long, env = "PP_CORS_ALLOW_ORIGIN", default_value = "*")]
  pub cors_allow_origin: String,

  /// CORS max-age in seconds [env: PP_CORS_MAX_AGE_SECS]
  #[arg(long, env = "PP_CORS_MAX_AGE_SECS", default_value = "600")]
  pub cors_max_age_secs: u64,

  /// Comma-separated input formats to block (jpeg,png,gif,webp,avif,jxl,bmp,tiff,pdf,psd,video) [env: PP_INPUT_DISALLOW_LIST]
  #[arg(long, env = "PP_INPUT_DISALLOW_LIST", default_value = "")]
  pub input_disallow_list: String,

  /// Comma-separated output formats to block (jpeg,png,gif,webp,avif,jxl,bmp,tiff,ico) [env: PP_OUTPUT_DISALLOW_LIST]
  #[arg(long, env = "PP_OUTPUT_DISALLOW_LIST", default_value = "")]
  pub output_disallow_list: String,

  /// Comma-separated transforms to block (resize,rotate,flip,grayscale,brightness,contrast,blur,watermark,gif_anim) [env: PP_TRANSFORM_DISALLOW_LIST]
  #[arg(long, env = "PP_TRANSFORM_DISALLOW_LIST", default_value = "")]
  pub transform_disallow_list: String,

  /// URL alias definitions: name=https://base.url,name2=https://other.url; enables name:/path scheme in requests [env: PP_URL_ALIASES]
  #[arg(long, env = "PP_URL_ALIASES", default_value = "")]
  pub url_aliases: String,

  /// Max in-flight requests before returning 503 [env: PP_MAX_CONCURRENT_REQUESTS]
  #[arg(long, env = "PP_MAX_CONCURRENT_REQUESTS", default_value_t = 256)]
  pub max_concurrent_requests: u32,

  /// Log level filter (e.g. previewproxy=info,tower_http=info) [env: RUST_LOG]
  #[arg(
    long,
    env = "RUST_LOG",
    default_value = "previewproxy=info,tower_http=info"
  )]
  pub rust_log: String,

  /// Hex-encoded AES key for encrypting/decrypting source URLs (leave empty to disable) [env: PP_SOURCE_URL_ENCRYPTION_KEY]
  #[arg(long, env = "PP_SOURCE_URL_ENCRYPTION_KEY")]
  pub source_url_encryption_key: Option<String>,

  /// Enable S3 as an image source [env: PP_S3_ENABLED]
  #[arg(long, env = "PP_S3_ENABLED", default_value_t = false)]
  pub s3_enabled: bool,

  /// S3 bucket name [env: PP_S3_BUCKET]
  #[arg(long, env = "PP_S3_BUCKET", default_value = "")]
  pub s3_bucket: String,

  /// S3 region [env: PP_S3_REGION]
  #[arg(long, env = "PP_S3_REGION", default_value = "us-east-1")]
  pub s3_region: String,

  /// S3 access key ID [env: PP_S3_ACCESS_KEY_ID]
  #[arg(long, env = "PP_S3_ACCESS_KEY_ID", default_value = "")]
  pub s3_access_key_id: String,

  /// S3 secret access key [env: PP_S3_SECRET_ACCESS_KEY]
  #[arg(long, env = "PP_S3_SECRET_ACCESS_KEY", default_value = "")]
  pub s3_secret_access_key: String,

  /// S3 custom endpoint URL (leave empty for AWS) [env: PP_S3_ENDPOINT]
  #[arg(long, env = "PP_S3_ENDPOINT", default_value = "")]
  pub s3_endpoint: String,

  /// Enable serving images from the local filesystem [env: PP_LOCAL_ENABLED]
  #[arg(long, env = "PP_LOCAL_ENABLED", default_value_t = false)]
  pub local_enabled: bool,

  /// Absolute path to root directory for local image files [env: PP_LOCAL_BASE_DIR]
  #[arg(long, env = "PP_LOCAL_BASE_DIR", default_value = "")]
  pub local_base_dir: String,

  /// Edge density threshold (0-100) for best-format complexity classification [env: PP_BEST_FORMAT_COMPLEXITY_THRESHOLD]
  #[arg(
    long,
    env = "PP_BEST_FORMAT_COMPLEXITY_THRESHOLD",
    default_value_t = 5.5
  )]
  pub best_format_complexity_threshold: f64,

  /// Max resolution in megapixels before skipping multi-format trial (leave empty to always trial) [env: PP_BEST_FORMAT_MAX_RESOLUTION]
  #[arg(long, env = "PP_BEST_FORMAT_MAX_RESOLUTION", default_value = "")]
  pub best_format_max_resolution: String,

  /// Apply best-format selection for all requests that don't specify a format [env: PP_BEST_FORMAT_BY_DEFAULT]
  #[arg(long, env = "PP_BEST_FORMAT_BY_DEFAULT", default_value_t = false)]
  pub best_format_by_default: bool,

  /// Skip re-encoding if selected best format matches source format and no transforms applied [env: PP_BEST_FORMAT_ALLOW_SKIPS]
  #[arg(long, env = "PP_BEST_FORMAT_ALLOW_SKIPS", default_value_t = false)]
  pub best_format_allow_skips: bool,

  /// Comma-separated formats to trial for best-format selection [env: PP_BEST_FORMAT_PREFERRED_FORMATS]
  #[arg(
    long,
    env = "PP_BEST_FORMAT_PREFERRED_FORMATS",
    default_value = "jpeg,webp,png"
  )]
  pub best_format_preferred_formats: String,

  /// Address to expose Prometheus metrics (e.g. :9464); leave empty to disable [env: PP_PROMETHEUS_BIND]
  #[arg(long, env = "PP_PROMETHEUS_BIND", default_value = "")]
  pub prometheus_bind: String,

  /// Prefix for all Prometheus metric names [env: PP_PROMETHEUS_NAMESPACE]
  #[arg(long, env = "PP_PROMETHEUS_NAMESPACE", default_value = "")]
  pub prometheus_namespace: String,

  /// Base64-encoded fallback image data [env: PP_FALLBACK_IMAGE_DATA]
  #[arg(long, env = "PP_FALLBACK_IMAGE_DATA")]
  pub fallback_image_data: Option<String>,

  /// Path to local fallback image file [env: PP_FALLBACK_IMAGE_PATH]
  #[arg(long, env = "PP_FALLBACK_IMAGE_PATH", default_value = "")]
  pub fallback_image_path: String,

  /// URL of fallback image [env: PP_FALLBACK_IMAGE_URL]
  #[arg(long, env = "PP_FALLBACK_IMAGE_URL", default_value = "")]
  pub fallback_image_url: String,

  /// HTTP status code for fallback responses; 0 = use original error code [env: PP_FALLBACK_IMAGE_HTTP_CODE]
  #[arg(long, env = "PP_FALLBACK_IMAGE_HTTP_CODE", default_value_t = 200u16)]
  pub fallback_image_http_code: u16,

  /// TTL in seconds for fallback image responses; 0 = use PP_TTL [env: PP_FALLBACK_IMAGE_TTL]
  #[arg(long, env = "PP_FALLBACK_IMAGE_TTL", default_value_t = 0u64)]
  pub fallback_image_ttl: u64,

  #[command(subcommand)]
  pub command: Option<Commands>,
}

impl Cli {
  pub fn apply_to_env(&self) {
    unsafe {
      std::env::set_var("PP_PORT", self.port.to_string());
      std::env::set_var("PP_APP_ENV", &self.env);
      std::env::set_var("PP_HMAC_KEY", self.hmac_key.as_deref().unwrap_or(""));
      std::env::set_var("PP_ALLOWED_HOSTS", &self.allowed_hosts);
      std::env::set_var("PP_FETCH_TIMEOUT_SECS", self.fetch_timeout_secs.to_string());
      std::env::set_var("PP_MAX_SOURCE_BYTES", self.max_source_bytes.to_string());
      std::env::set_var(
        "PP_CACHE_MEMORY_MAX_MB",
        self.cache_memory_max_mb.to_string(),
      );
      std::env::set_var(
        "PP_CACHE_MEMORY_TTL_SECS",
        self.cache_memory_ttl_secs.to_string(),
      );
      std::env::set_var("PP_CACHE_DIR", &self.cache_dir);
      std::env::set_var(
        "PP_CACHE_DISK_TTL_SECS",
        self.cache_disk_ttl_secs.to_string(),
      );
      std::env::set_var("PP_CACHE_DISK_MAX_MB", &self.cache_disk_max_mb);
      std::env::set_var(
        "PP_CACHE_CLEANUP_INTERVAL_SECS",
        self.cache_cleanup_interval_secs.to_string(),
      );
      std::env::set_var("PP_FFMPEG_PATH", &self.ffmpeg_path);
      std::env::set_var("PP_FFPROBE_PATH", &self.ffprobe_path);
      std::env::set_var("PP_CORS_ALLOW_ORIGIN", &self.cors_allow_origin);
      std::env::set_var("PP_CORS_MAX_AGE_SECS", self.cors_max_age_secs.to_string());
      std::env::set_var("PP_INPUT_DISALLOW_LIST", &self.input_disallow_list);
      std::env::set_var("PP_OUTPUT_DISALLOW_LIST", &self.output_disallow_list);
      std::env::set_var("PP_TRANSFORM_DISALLOW_LIST", &self.transform_disallow_list);
      std::env::set_var("PP_URL_ALIASES", &self.url_aliases);
      std::env::set_var(
        "PP_MAX_CONCURRENT_REQUESTS",
        self.max_concurrent_requests.to_string(),
      );
      std::env::set_var("RUST_LOG", &self.rust_log);
      std::env::set_var(
        "PP_SOURCE_URL_ENCRYPTION_KEY",
        self.source_url_encryption_key.as_deref().unwrap_or(""),
      );
      std::env::set_var("PP_S3_ENABLED", self.s3_enabled.to_string());
      std::env::set_var("PP_S3_BUCKET", &self.s3_bucket);
      std::env::set_var("PP_S3_REGION", &self.s3_region);
      std::env::set_var("PP_S3_ACCESS_KEY_ID", &self.s3_access_key_id);
      std::env::set_var("PP_S3_SECRET_ACCESS_KEY", &self.s3_secret_access_key);
      std::env::set_var("PP_S3_ENDPOINT", &self.s3_endpoint);
      std::env::set_var("PP_LOCAL_ENABLED", self.local_enabled.to_string());
      std::env::set_var("PP_LOCAL_BASE_DIR", &self.local_base_dir);
      std::env::set_var(
        "PP_BEST_FORMAT_COMPLEXITY_THRESHOLD",
        self.best_format_complexity_threshold.to_string(),
      );
      std::env::set_var(
        "PP_BEST_FORMAT_MAX_RESOLUTION",
        &self.best_format_max_resolution,
      );
      std::env::set_var(
        "PP_BEST_FORMAT_BY_DEFAULT",
        self.best_format_by_default.to_string(),
      );
      std::env::set_var(
        "PP_BEST_FORMAT_ALLOW_SKIPS",
        self.best_format_allow_skips.to_string(),
      );
      std::env::set_var(
        "PP_BEST_FORMAT_PREFERRED_FORMATS",
        &self.best_format_preferred_formats,
      );
      std::env::set_var("PP_PROMETHEUS_BIND", &self.prometheus_bind);
      std::env::set_var("PP_PROMETHEUS_NAMESPACE", &self.prometheus_namespace);
      std::env::set_var(
        "PP_FALLBACK_IMAGE_DATA",
        self.fallback_image_data.as_deref().unwrap_or(""),
      );
      std::env::set_var("PP_FALLBACK_IMAGE_PATH", &self.fallback_image_path);
      std::env::set_var("PP_FALLBACK_IMAGE_URL", &self.fallback_image_url);
      std::env::set_var(
        "PP_FALLBACK_IMAGE_HTTP_CODE",
        self.fallback_image_http_code.to_string(),
      );
      std::env::set_var("PP_FALLBACK_IMAGE_TTL", self.fallback_image_ttl.to_string());
      std::env::set_var("PP_TTL", self.ttl.to_string());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::Mutex;
  static ENV_LOCK: Mutex<()> = Mutex::new(());

  fn base_args() -> Vec<&'static str> {
    vec!["previewproxy"]
  }

  #[test]
  fn test_max_concurrent_requests_default() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { std::env::remove_var("PP_MAX_CONCURRENT_REQUESTS") };
    let cli = Cli::parse_from(base_args());
    assert_eq!(cli.max_concurrent_requests, 256);
  }

  #[test]
  fn test_max_concurrent_requests_arg() {
    let cli = Cli::parse_from(["previewproxy", "--max-concurrent-requests", "64"]);
    assert_eq!(cli.max_concurrent_requests, 64);
  }

  #[test]
  fn test_rust_log_default() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { std::env::remove_var("RUST_LOG") };
    let cli = Cli::parse_from(base_args());
    assert_eq!(cli.rust_log, "previewproxy=info,tower_http=info");
  }

  #[test]
  fn test_source_url_encryption_key_absent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { std::env::remove_var("PP_SOURCE_URL_ENCRYPTION_KEY") };
    let cli = Cli::parse_from(base_args());
    assert!(cli.source_url_encryption_key.is_none());
  }

  #[test]
  fn test_source_url_encryption_key_arg() {
    let cli = Cli::parse_from(["previewproxy", "--source-url-encryption-key", "abc123"]);
    assert_eq!(cli.source_url_encryption_key.as_deref(), Some("abc123"));
  }

  #[test]
  fn test_s3_defaults() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
      std::env::remove_var("PP_S3_ENABLED");
      std::env::remove_var("PP_S3_BUCKET");
      std::env::remove_var("PP_S3_REGION");
      std::env::remove_var("PP_S3_ACCESS_KEY_ID");
      std::env::remove_var("PP_S3_SECRET_ACCESS_KEY");
      std::env::remove_var("PP_S3_ENDPOINT");
    }
    let cli = Cli::parse_from(base_args());
    assert!(!cli.s3_enabled);
    assert_eq!(cli.s3_bucket, "");
    assert_eq!(cli.s3_region, "us-east-1");
    assert_eq!(cli.s3_access_key_id, "");
    assert_eq!(cli.s3_secret_access_key, "");
    assert_eq!(cli.s3_endpoint, "");
  }

  #[test]
  fn test_s3_args() {
    let cli = Cli::parse_from([
      "previewproxy",
      "--s3-enabled",
      "--s3-bucket",
      "mybucket",
      "--s3-region",
      "eu-west-1",
      "--s3-access-key-id",
      "keyid",
      "--s3-secret-access-key",
      "secret",
      "--s3-endpoint",
      "https://r2.example.com",
    ]);
    assert!(cli.s3_enabled);
    assert_eq!(cli.s3_bucket, "mybucket");
    assert_eq!(cli.s3_region, "eu-west-1");
    assert_eq!(cli.s3_access_key_id, "keyid");
    assert_eq!(cli.s3_secret_access_key, "secret");
    assert_eq!(cli.s3_endpoint, "https://r2.example.com");
  }

  #[test]
  fn test_local_defaults() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
      std::env::remove_var("PP_LOCAL_ENABLED");
      std::env::remove_var("PP_LOCAL_BASE_DIR");
    }
    let cli = Cli::parse_from(base_args());
    assert!(!cli.local_enabled);
    assert_eq!(cli.local_base_dir, "");
  }

  #[test]
  fn test_local_args() {
    let cli = Cli::parse_from([
      "previewproxy",
      "--local-enabled",
      "--local-base-dir",
      "/data/images",
    ]);
    assert!(cli.local_enabled);
    assert_eq!(cli.local_base_dir, "/data/images");
  }

  #[test]
  fn test_best_format_defaults() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
      std::env::remove_var("PP_BEST_FORMAT_COMPLEXITY_THRESHOLD");
      std::env::remove_var("PP_BEST_FORMAT_MAX_RESOLUTION");
      std::env::remove_var("PP_BEST_FORMAT_BY_DEFAULT");
      std::env::remove_var("PP_BEST_FORMAT_ALLOW_SKIPS");
      std::env::remove_var("PP_BEST_FORMAT_PREFERRED_FORMATS");
    }
    let cli = Cli::parse_from(base_args());
    assert!((cli.best_format_complexity_threshold - 5.5).abs() < f64::EPSILON);
    assert_eq!(cli.best_format_max_resolution, "");
    assert!(!cli.best_format_by_default);
    assert!(!cli.best_format_allow_skips);
    assert_eq!(cli.best_format_preferred_formats, "jpeg,webp,png");
  }

  #[test]
  fn test_best_format_args() {
    let cli = Cli::parse_from([
      "previewproxy",
      "--best-format-complexity-threshold",
      "8.0",
      "--best-format-max-resolution",
      "4.0",
      "--best-format-by-default",
      "--best-format-allow-skips",
      "--best-format-preferred-formats",
      "webp,avif",
    ]);
    assert!((cli.best_format_complexity_threshold - 8.0).abs() < f64::EPSILON);
    assert_eq!(cli.best_format_max_resolution, "4.0");
    assert!(cli.best_format_by_default);
    assert!(cli.best_format_allow_skips);
    assert_eq!(cli.best_format_preferred_formats, "webp,avif");
  }

  #[test]
  fn test_apply_to_env_new_fields() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let cli = Cli::parse_from([
      "previewproxy",
      "--max-concurrent-requests",
      "128",
      "--s3-enabled",
      "--s3-bucket",
      "testbucket",
      "--local-enabled",
      "--local-base-dir",
      "/srv/images",
      "--best-format-by-default",
      "--best-format-preferred-formats",
      "webp,avif",
      "--source-url-encryption-key",
      "hexkey",
    ]);
    cli.apply_to_env();
    assert_eq!(std::env::var("PP_MAX_CONCURRENT_REQUESTS").unwrap(), "128");
    assert_eq!(std::env::var("PP_S3_ENABLED").unwrap(), "true");
    assert_eq!(std::env::var("PP_S3_BUCKET").unwrap(), "testbucket");
    assert_eq!(std::env::var("PP_LOCAL_ENABLED").unwrap(), "true");
    assert_eq!(std::env::var("PP_LOCAL_BASE_DIR").unwrap(), "/srv/images");
    assert_eq!(std::env::var("PP_BEST_FORMAT_BY_DEFAULT").unwrap(), "true");
    assert_eq!(
      std::env::var("PP_BEST_FORMAT_PREFERRED_FORMATS").unwrap(),
      "webp,avif"
    );
    assert_eq!(
      std::env::var("PP_SOURCE_URL_ENCRYPTION_KEY").unwrap(),
      "hexkey"
    );
  }

  #[test]
  fn test_fallback_image_cli_defaults() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
      std::env::remove_var("PP_FALLBACK_IMAGE_DATA");
      std::env::remove_var("PP_FALLBACK_IMAGE_PATH");
      std::env::remove_var("PP_FALLBACK_IMAGE_URL");
      std::env::remove_var("PP_FALLBACK_IMAGE_HTTP_CODE");
      std::env::remove_var("PP_FALLBACK_IMAGE_TTL");
      std::env::remove_var("PP_TTL");
    }
    let cli = Cli::parse_from(["previewproxy"]);
    assert!(cli.fallback_image_data.is_none());
    assert_eq!(cli.fallback_image_path, "");
    assert_eq!(cli.fallback_image_url, "");
    assert_eq!(cli.fallback_image_http_code, 200u16);
    assert_eq!(cli.fallback_image_ttl, 0u64);
    assert_eq!(cli.ttl, 86400u64);
  }
}
