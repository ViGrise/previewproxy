use crate::modules::cli::dto::Commands;
use clap::Parser;

/// ViGrise PreviewProxy - on-the-fly image proxy and transformer
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Path to a .env file to load (overrides default .env lookup)
  #[arg(long, value_name = "PATH", global = true)]
  pub env_file: Option<String>,

  /// Server port [env: PORT]
  #[arg(short, long, env = "PORT", default_value = "8080")]
  pub port: u16,

  /// Environment: development or production [env: APP_ENV]
  #[arg(short = 'E', long, env = "APP_ENV", default_value = "development")]
  pub env: String,

  /// HMAC signing key (leave empty to disable) [env: HMAC_KEY]
  #[arg(short = 'k', long, env = "HMAC_KEY")]
  pub hmac_key: Option<String>,

  /// Comma-separated allowed upstream hosts (empty = allow all) [env: ALLOWED_HOSTS]
  #[arg(short = 'a', long, env = "ALLOWED_HOSTS", default_value = "")]
  pub allowed_hosts: String,

  /// Upstream fetch timeout in seconds [env: FETCH_TIMEOUT_SECS]
  #[arg(short = 't', long, env = "FETCH_TIMEOUT_SECS", default_value = "10")]
  pub fetch_timeout_secs: u64,

  /// Maximum source image size in bytes [env: MAX_SOURCE_BYTES]
  #[arg(
    short = 's',
    long,
    env = "MAX_SOURCE_BYTES",
    default_value = "20971520"
  )]
  pub max_source_bytes: u64,

  /// L1 in-memory cache size in MB [env: CACHE_MEMORY_MAX_MB]
  #[arg(long, env = "CACHE_MEMORY_MAX_MB", default_value = "256")]
  pub cache_memory_max_mb: u64,

  /// L1 in-memory cache TTL in seconds [env: CACHE_MEMORY_TTL_SECS]
  #[arg(long, env = "CACHE_MEMORY_TTL_SECS", default_value = "3600")]
  pub cache_memory_ttl_secs: u64,

  /// L2 disk cache directory [env: CACHE_DIR]
  #[arg(
    short = 'D',
    long,
    env = "CACHE_DIR",
    default_value = "/tmp/previewproxy"
  )]
  pub cache_dir: String,

  /// L2 disk cache TTL in seconds [env: CACHE_DISK_TTL_SECS]
  #[arg(long, env = "CACHE_DISK_TTL_SECS", default_value = "86400")]
  pub cache_disk_ttl_secs: u64,

  /// L2 disk cache max size in MB (empty = unlimited) [env: CACHE_DISK_MAX_MB]
  #[arg(long, env = "CACHE_DISK_MAX_MB", default_value = "")]
  pub cache_disk_max_mb: String,

  /// Cache cleanup interval in seconds [env: CACHE_CLEANUP_INTERVAL_SECS]
  #[arg(long, env = "CACHE_CLEANUP_INTERVAL_SECS", default_value = "600")]
  pub cache_cleanup_interval_secs: u64,

  /// Path to the ffmpeg binary [env: FFMPEG_PATH]
  #[arg(long, env = "FFMPEG_PATH", default_value = "ffmpeg")]
  pub ffmpeg_path: String,

  /// Path to the ffprobe binary (defaults to ffprobe in same dir as ffmpeg) [env: FFPROBE_PATH]
  #[arg(long, env = "FFPROBE_PATH", default_value = "")]
  pub ffprobe_path: String,

  /// Comma-separated allowed CORS origins; * = allow all [env: CORS_ALLOW_ORIGIN]
  #[arg(long, env = "CORS_ALLOW_ORIGIN", default_value = "*")]
  pub cors_allow_origin: String,

  /// CORS max-age in seconds [env: CORS_MAX_AGE_SECS]
  #[arg(long, env = "CORS_MAX_AGE_SECS", default_value = "600")]
  pub cors_max_age_secs: u64,

  /// Comma-separated input formats to block (jpeg,png,gif,webp,avif,jxl,bmp,tiff,pdf,psd,video) [env: INPUT_DISALLOW_LIST]
  #[arg(long, env = "INPUT_DISALLOW_LIST", default_value = "")]
  pub input_disallow_list: String,

  /// Comma-separated output formats to block (jpeg,png,gif,webp,avif,jxl,bmp,tiff,ico) [env: OUTPUT_DISALLOW_LIST]
  #[arg(long, env = "OUTPUT_DISALLOW_LIST", default_value = "")]
  pub output_disallow_list: String,

  /// Comma-separated transforms to block (resize,rotate,flip,grayscale,brightness,contrast,blur,watermark,gif_anim) [env: TRANSFORM_DISALLOW_LIST]
  #[arg(long, env = "TRANSFORM_DISALLOW_LIST", default_value = "")]
  pub transform_disallow_list: String,

  /// URL alias definitions: name=https://base.url,name2=https://other.url; enables name:/path scheme in requests [env: URL_ALIASES]
  #[arg(long, env = "URL_ALIASES", default_value = "")]
  pub url_aliases: String,

  /// Max in-flight requests before returning 503 [env: MAX_CONCURRENT_REQUESTS]
  #[arg(long, env = "MAX_CONCURRENT_REQUESTS", default_value_t = 256)]
  pub max_concurrent_requests: u32,

  /// Log level filter (e.g. previewproxy=info,tower_http=info) [env: RUST_LOG]
  #[arg(
    long,
    env = "RUST_LOG",
    default_value = "previewproxy=info,tower_http=info"
  )]
  pub rust_log: String,

  /// Hex-encoded AES key for encrypting/decrypting source URLs (leave empty to disable) [env: SOURCE_URL_ENCRYPTION_KEY]
  #[arg(long, env = "SOURCE_URL_ENCRYPTION_KEY")]
  pub source_url_encryption_key: Option<String>,

  /// Enable S3 as an image source [env: S3_ENABLED]
  #[arg(long, env = "S3_ENABLED", default_value_t = false)]
  pub s3_enabled: bool,

  /// S3 bucket name [env: S3_BUCKET]
  #[arg(long, env = "S3_BUCKET", default_value = "")]
  pub s3_bucket: String,

  /// S3 region [env: S3_REGION]
  #[arg(long, env = "S3_REGION", default_value = "us-east-1")]
  pub s3_region: String,

  /// S3 access key ID [env: S3_ACCESS_KEY_ID]
  #[arg(long, env = "S3_ACCESS_KEY_ID", default_value = "")]
  pub s3_access_key_id: String,

  /// S3 secret access key [env: S3_SECRET_ACCESS_KEY]
  #[arg(long, env = "S3_SECRET_ACCESS_KEY", default_value = "")]
  pub s3_secret_access_key: String,

  /// S3 custom endpoint URL (leave empty for AWS) [env: S3_ENDPOINT]
  #[arg(long, env = "S3_ENDPOINT", default_value = "")]
  pub s3_endpoint: String,

  /// Enable serving images from the local filesystem [env: LOCAL_ENABLED]
  #[arg(long, env = "LOCAL_ENABLED", default_value_t = false)]
  pub local_enabled: bool,

  /// Absolute path to root directory for local image files [env: LOCAL_BASE_DIR]
  #[arg(long, env = "LOCAL_BASE_DIR", default_value = "")]
  pub local_base_dir: String,

  /// Edge density threshold (0-100) for best-format complexity classification [env: BEST_FORMAT_COMPLEXITY_THRESHOLD]
  #[arg(long, env = "BEST_FORMAT_COMPLEXITY_THRESHOLD", default_value_t = 5.5)]
  pub best_format_complexity_threshold: f64,

  /// Max resolution in megapixels before skipping multi-format trial (leave empty to always trial) [env: BEST_FORMAT_MAX_RESOLUTION]
  #[arg(long, env = "BEST_FORMAT_MAX_RESOLUTION", default_value = "")]
  pub best_format_max_resolution: String,

  /// Apply best-format selection for all requests that don't specify a format [env: BEST_FORMAT_BY_DEFAULT]
  #[arg(long, env = "BEST_FORMAT_BY_DEFAULT", default_value_t = false)]
  pub best_format_by_default: bool,

  /// Skip re-encoding if selected best format matches source format and no transforms applied [env: BEST_FORMAT_ALLOW_SKIPS]
  #[arg(long, env = "BEST_FORMAT_ALLOW_SKIPS", default_value_t = false)]
  pub best_format_allow_skips: bool,

  /// Comma-separated formats to trial for best-format selection [env: BEST_FORMAT_PREFERRED_FORMATS]
  #[arg(
    long,
    env = "BEST_FORMAT_PREFERRED_FORMATS",
    default_value = "jpeg,webp,png"
  )]
  pub best_format_preferred_formats: String,

  #[command(subcommand)]
  pub command: Option<Commands>,
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
    let cli = Cli::parse_from(base_args());
    assert_eq!(cli.rust_log, "previewproxy=info,tower_http=info");
  }

  #[test]
  fn test_source_url_encryption_key_absent() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::remove_var("SOURCE_URL_ENCRYPTION_KEY") };
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
    let _guard = ENV_LOCK.lock().unwrap();
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
    assert_eq!(std::env::var("MAX_CONCURRENT_REQUESTS").unwrap(), "128");
    assert_eq!(std::env::var("S3_ENABLED").unwrap(), "true");
    assert_eq!(std::env::var("S3_BUCKET").unwrap(), "testbucket");
    assert_eq!(std::env::var("LOCAL_ENABLED").unwrap(), "true");
    assert_eq!(std::env::var("LOCAL_BASE_DIR").unwrap(), "/srv/images");
    assert_eq!(std::env::var("BEST_FORMAT_BY_DEFAULT").unwrap(), "true");
    assert_eq!(
      std::env::var("BEST_FORMAT_PREFERRED_FORMATS").unwrap(),
      "webp,avif"
    );
    assert_eq!(
      std::env::var("SOURCE_URL_ENCRYPTION_KEY").unwrap(),
      "hexkey"
    );
  }
}

impl Cli {
  pub fn apply_to_env(&self) {
    unsafe {
      std::env::set_var("PORT", self.port.to_string());
      std::env::set_var("APP_ENV", &self.env);
      std::env::set_var("HMAC_KEY", self.hmac_key.as_deref().unwrap_or(""));
      std::env::set_var("ALLOWED_HOSTS", &self.allowed_hosts);
      std::env::set_var("FETCH_TIMEOUT_SECS", self.fetch_timeout_secs.to_string());
      std::env::set_var("MAX_SOURCE_BYTES", self.max_source_bytes.to_string());
      std::env::set_var("CACHE_MEMORY_MAX_MB", self.cache_memory_max_mb.to_string());
      std::env::set_var(
        "CACHE_MEMORY_TTL_SECS",
        self.cache_memory_ttl_secs.to_string(),
      );
      std::env::set_var("CACHE_DIR", &self.cache_dir);
      std::env::set_var("CACHE_DISK_TTL_SECS", self.cache_disk_ttl_secs.to_string());
      std::env::set_var("CACHE_DISK_MAX_MB", &self.cache_disk_max_mb);
      std::env::set_var(
        "CACHE_CLEANUP_INTERVAL_SECS",
        self.cache_cleanup_interval_secs.to_string(),
      );
      std::env::set_var("FFMPEG_PATH", &self.ffmpeg_path);
      std::env::set_var("FFPROBE_PATH", &self.ffprobe_path);
      std::env::set_var("CORS_ALLOW_ORIGIN", &self.cors_allow_origin);
      std::env::set_var("CORS_MAX_AGE_SECS", self.cors_max_age_secs.to_string());
      std::env::set_var("INPUT_DISALLOW_LIST", &self.input_disallow_list);
      std::env::set_var("OUTPUT_DISALLOW_LIST", &self.output_disallow_list);
      std::env::set_var("TRANSFORM_DISALLOW_LIST", &self.transform_disallow_list);
      std::env::set_var("URL_ALIASES", &self.url_aliases);
      std::env::set_var(
        "MAX_CONCURRENT_REQUESTS",
        self.max_concurrent_requests.to_string(),
      );
      std::env::set_var("RUST_LOG", &self.rust_log);
      std::env::set_var(
        "SOURCE_URL_ENCRYPTION_KEY",
        self.source_url_encryption_key.as_deref().unwrap_or(""),
      );
      std::env::set_var("S3_ENABLED", self.s3_enabled.to_string());
      std::env::set_var("S3_BUCKET", &self.s3_bucket);
      std::env::set_var("S3_REGION", &self.s3_region);
      std::env::set_var("S3_ACCESS_KEY_ID", &self.s3_access_key_id);
      std::env::set_var("S3_SECRET_ACCESS_KEY", &self.s3_secret_access_key);
      std::env::set_var("S3_ENDPOINT", &self.s3_endpoint);
      std::env::set_var("LOCAL_ENABLED", self.local_enabled.to_string());
      std::env::set_var("LOCAL_BASE_DIR", &self.local_base_dir);
      std::env::set_var(
        "BEST_FORMAT_COMPLEXITY_THRESHOLD",
        self.best_format_complexity_threshold.to_string(),
      );
      std::env::set_var(
        "BEST_FORMAT_MAX_RESOLUTION",
        &self.best_format_max_resolution,
      );
      std::env::set_var(
        "BEST_FORMAT_BY_DEFAULT",
        self.best_format_by_default.to_string(),
      );
      std::env::set_var(
        "BEST_FORMAT_ALLOW_SKIPS",
        self.best_format_allow_skips.to_string(),
      );
      std::env::set_var(
        "BEST_FORMAT_PREFERRED_FORMATS",
        &self.best_format_preferred_formats,
      );
    }
  }
}
